use crate::filesystem::temp::TempFile;
use crate::filesystem::FileSystem;
use crate::process::event::EventHandler;
use crate::process::tracee::Tracee;
use std::ffi::CString;
use std::{collections::HashMap, convert::TryFrom};

// libc
use libc::{c_int, c_void, pid_t, siginfo_t};
// signals
use nix::sys::{
    signal::{kill, Signal},
    wait::WaitPidFlag,
};
use nix::unistd::Pid;
// ptrace
use nix::sys::ptrace;
// fork
use nix::unistd::{execvp, fork, getpid, ForkResult};
// event loop
use nix::sys::ptrace::Event as PtraceEvent;
use nix::sys::wait;
use nix::sys::wait::WaitStatus::*;

/// Used to store global info common to all tracees. Rename into `Configuration`?
#[derive(Debug)]
pub struct InfoBag {
    /// Used to know if the first raw sigtrap has been processed
    /// (and if the `set_ptrace_options` step is required).
    pub deliver_sigtrap: bool,
    /// Binary loader, used by `execve`.
    /// The content of the binary is actually inlined in `proot-rs`
    /// (see `src/kernel/execve/loader`), and is extracted into a temporary file before use.
    /// This temporary file struct makes sure the file is deleted when it's dropped.
    pub loader: TempFile,
}

impl InfoBag {
    pub fn new() -> InfoBag {
        InfoBag {
            deliver_sigtrap: false,
            loader: TempFile::new("prooted"),
        }
    }
}

#[derive(Debug)]
pub struct PRoot {
    info_bag: InfoBag,
    tracees: HashMap<Pid, Tracee>,
    alive_tracees: Vec<Pid>,
}

impl PRoot {
    pub fn new() -> PRoot {
        PRoot {
            info_bag: InfoBag::new(),
            tracees: HashMap::new(),
            alive_tracees: vec![],
        }
    }

    /// Main process where proot splits into two threads:
    /// - a tracer, the parent thread.
    /// - a (first) tracee, the child thread,
    ///   that will declare itself as ptrace-able before executing the program.
    ///
    /// The `fork()` done here implies that the OS will apply copy-on-write
    /// on all the shared memory of the parent and child processes
    /// (heap, libraries...), so both of them will have their own (owned) version
    /// of the PRoot memory.
    pub fn launch_process(&mut self, initial_fs: FileSystem) {
        match unsafe { fork() }.expect("fork in launch process") {
            ForkResult::Parent { child } => {
                // we create the first tracee
                self.create_tracee(child, initial_fs);
            }
            ForkResult::Child => {
                // Declare the tracee as ptraceable
                ptrace::traceme().expect("ptrace traceme");

                // Synchronise with the parent's event loop by waiting until it's ready
                // (otherwise the execvp is executed too quickly)
                kill(getpid(), Signal::SIGSTOP).expect("first child synchronisation");

                //TODO: seccomp
                //if (getenv("PROOT_NO_SECCOMP") == NULL)
                //    (void) enable_syscall_filtering(tracee);

                execvp(
                    &CString::new("sleep").unwrap(),
                    &[CString::new(".").unwrap(), CString::new("0").unwrap()],
                )
                .expect("failed execvp sleep");
                //execvp(&CString::new("echo").unwrap(), &[CString::new(".").unwrap(),
                // CString::new("TRACEE ECHO").unwrap()])
                //    .expect("failed execvp echo");
                //execvp(&CString::new("ls").unwrap(), &[CString::new(".").unwrap()])
                //   .expect("failed execvp ls");
                //TODO: cli must handle command, or use 'sh' as default (like proot)
                //execvp(tracee->exe, argv[0] != NULL ? argv : default_argv);
            }
        }
    }

    /// Infinite loop where PRoot will wait for tracees signals with `waitpid`.
    /// Tracees will be stopped when they use a system call.
    /// The tracer will be notified through `waitpid` and will be able to alter
    /// the parameters of the system call, before restarting the tracee.
    pub fn event_loop(&mut self) {
        while !self.alive_tracees.is_empty() {
            match wait::waitpid(Pid::from_raw(-1), Some(WaitPidFlag::__WALL))
                .expect("event loop waitpid")
            {
                Exited(pid, exit_status) => {
                    println!("-- {}, Exited with status: {}", pid, exit_status);
                    self.register_tracee_finished(pid);
                }
                Signaled(pid, term_signal, dumped_core) => {
                    println!(
                        "-- {}, Signaled with status: {:?}, and dump core: {}",
                        pid, term_signal, dumped_core
                    );
                    self.register_tracee_finished(pid);
                }
                // The tracee was stopped by a normal signal (signal-delivery-stop), or was stopped
                // by a system call (syscall-stop) with PTRACE_O_TRACESYSGOOD not effect. We can
                // use PTRACE_GETSIGINFO to distinguish the second situation.
                Stopped(pid, stop_signal) => {
                    println!(
                        "-- {}, Stopped, {:?}, {}",
                        pid, stop_signal, stop_signal as c_int
                    );
                    let tracee = self.tracees.get_mut(&pid).expect("get stopped tracee");
                    tracee.reset_restart_how();
                    match stop_signal {
                        Signal::SIGSTOP => tracee.handle_sigstop_event(),
                        Signal::SIGTRAP => {
                            // it's the initial SIGTRAP signal
                            tracee.set_ptrace_options(&mut self.info_bag);
                            // Use PTRACE_GETSIGINFO to distinguish a real syscall-stop. see ptrace(2): Syscall-stops
                            if let Ok(siginfo) = ptrace::getsiginfo(pid) {
                                if siginfo.si_code == Signal::SIGTRAP as i32
                                    || siginfo.si_code == (Signal::SIGTRAP as i32 | 0x80)
                                {
                                    tracee.handle_syscall_stop_event(&mut self.info_bag);
                                }
                            }
                        }
                        _ => {}
                    }
                    // TODO: we should deliver this signal(sig) with ptrace(PTRACE_restart, pid, 0, sig)
                    tracee.restart();
                }
                // The tracee was stopped by a SIGTRAP with additional status (PTRACE_EVENT stops).
                // In this case, the status should be (SIGTRAP | PTRACE_EVENT_foo << 8).
                PtraceEvent(pid, signal, status_additional) => {
                    println!(
                        "-- {}, Ptrace event, {:?}, {:?}",
                        pid, signal, status_additional
                    );
                    let tracee = self.tracees.get_mut(&pid).expect("get stopped tracee");
                    tracee.reset_restart_how();

                    // handle_new_child_event
                    if status_additional == PtraceEvent::PTRACE_EVENT_VFORK as i32 {
                        tracee.handle_new_child_event(PtraceEvent::PTRACE_EVENT_VFORK);
                    } else if status_additional == PtraceEvent::PTRACE_EVENT_FORK as i32 {
                        tracee.handle_new_child_event(PtraceEvent::PTRACE_EVENT_FORK);
                    } else if status_additional == PtraceEvent::PTRACE_EVENT_CLONE as i32 {
                        tracee.handle_new_child_event(PtraceEvent::PTRACE_EVENT_CLONE);
                    }
                    // handle_exec_vfork_event
                    if status_additional == PtraceEvent::PTRACE_EVENT_EXEC as i32
                        || status_additional == PtraceEvent::PTRACE_EVENT_VFORK_DONE as i32
                    {
                        tracee.handle_exec_vfork_event();
                    }
                    // handle_seccomp_event
                    if status_additional == PtraceEvent::PTRACE_EVENT_SECCOMP as i32 {
                        // TODO: consider PTRACE_EVENT_SECCOMP2
                        tracee.handle_seccomp_event(
                            &mut self.info_bag,
                            PtraceEvent::PTRACE_EVENT_SECCOMP,
                        )
                    }
                    tracee.restart();
                }
                // The tracee was stopped by execution of a system call (syscall-stop), and
                // PTRACE_O_TRACESYSGOOD was effect. PTRACE_O_TRACESYSGOOD is used to make it
                // easy for the tracer to distinguish normal SIGTRAP from those caused by
                // a system call. In this case, the status should be (SIGTRAP | 0x80).
                PtraceSyscall(pid) => {
                    println!("-- {}, Syscall", pid);
                    let tracee = self.tracees.get_mut(&pid).expect("get stopped tracee");
                    tracee.reset_restart_how();
                    tracee.handle_syscall_stop_event(&mut self.info_bag);
                    tracee.restart();
                }
                Continued(pid) => {
                    println!("-- {}, Continued", pid);
                }
                StillAlive => {
                    println!("-- Still alive");
                }
            }
        }
    }

    /******** Utilities ****************/

    pub fn create_tracee(&mut self, pid: Pid, fs: FileSystem) -> Option<&Tracee> {
        self.tracees.insert(pid, Tracee::new(pid, fs));
        self.register_alive_tracee(pid);
        self.tracees.get(&pid)
    }

    fn register_alive_tracee(&mut self, pid: Pid) {
        self.alive_tracees.push(pid);
    }

    fn register_tracee_finished(&mut self, finished_pid: Pid) {
        self.alive_tracees.retain(|pid| *pid != finished_pid);
        self.tracees.remove(&finished_pid);
    }
}

/// Proot has received a fatal error from one of the tracee,
/// and must therefore stop the program's execution.
pub extern "C" fn stop_program(sig_num: c_int, _: *mut siginfo_t, _: *mut c_void) {
    let signal = Signal::try_from(sig_num);
    panic!("abnormal signal received: {:?}", signal);
}

pub extern "C" fn show_info(pid: pid_t) {
    println!("showing info pid {}", pid);
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix::unistd::Pid;

    #[test]
    fn create_proot_and_tracee() {
        let fs = FileSystem::new();
        let mut proot = PRoot::new();

        // tracee 0 shouldn't exist
        {
            let tracee = proot.tracees.get_mut(&Pid::from_raw(0));
            assert!(tracee.is_none());
        }

        {
            proot.create_tracee(Pid::from_raw(0), fs);
        }

        // tracee 0 should exist
        {
            let tracee = proot.tracees.get_mut(&Pid::from_raw(0));
            assert!(tracee.is_some());
        }
    }
}
