use filesystem::temp::TempFile;
use filesystem::FileSystem;
use process::event::EventHandler;
use process::tracee::Tracee;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr::null_mut;

// libc
use libc::{c_int, c_void, pid_t, siginfo_t};
// signals
use nix::sys::signal::{kill, Signal, SIGSTOP};
use nix::unistd::Pid;
// ptrace
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::PTRACE_TRACEME;
// fork
use nix::unistd::{execvp, fork, getpid, ForkResult};
// event loop
use nix::sys::wait::WaitStatus::*;
use nix::sys::wait::{waitpid, __WALL};

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
        match fork().expect("fork in launch process") {
            ForkResult::Parent { child } => {
                // we create the first tracee
                self.create_tracee(child, initial_fs);
            }
            ForkResult::Child => {
                // Declare the tracee as ptraceable
                ptrace(PTRACE_TRACEME, Pid::from_raw(0), null_mut(), null_mut())
                    .expect("ptrace traceme");

                // Synchronise with the parent's event loop by waiting until it's ready
                // (otherwise the execvp is executed too quickly)
                kill(getpid(), SIGSTOP).expect("first child synchronisation");

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
            match waitpid(Pid::from_raw(-1), Some(__WALL)).expect("event loop waitpid") {
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
                Stopped(pid, stop_signal) => {
                    println!(
                        "-- {}, Stopped, {:?}, {}",
                        pid, stop_signal, stop_signal as c_int
                    );
                    self.handle_standard_event(pid, Some(stop_signal));
                }
                PtraceEvent(pid, signal, additional_signal) => {
                    println!(
                        "-- {}, Ptrace event, {:?}, {:?}",
                        pid, signal, additional_signal
                    );
                    self.handle_standard_event(pid, Some(signal));
                }
                PtraceSyscall(pid) => {
                    //println!("-- {}, Syscall", pid);
                    self.handle_standard_event(pid, None);
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

    fn handle_standard_event(&mut self, tracee_pid: Pid, signal: Option<Signal>) {
        let (wrapped_tracee, info_bag) = self.get_mut_tracee_and_all(tracee_pid);
        let mut tracee = wrapped_tracee.expect("get stopped tracee");

        tracee.handle_event(info_bag, signal);
        tracee.restart();
    }

    /******** Utilities ****************/

    pub fn create_tracee(&mut self, pid: Pid, fs: FileSystem) -> Option<&Tracee> {
        self.tracees.insert(pid, Tracee::new(pid, fs));
        self.register_alive_tracee(pid);
        self.tracees.get(&pid)
    }

    fn get_mut_tracee_and_all(&mut self, pid: Pid) -> (Option<&mut Tracee>, &mut InfoBag) {
        (self.tracees.get_mut(&pid), &mut self.info_bag)
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
    let signal = Signal::from_c_int(sig_num).unwrap();
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
            let (tracee, _) = proot.get_mut_tracee_and_all(Pid::from_raw(0));
            assert!(tracee.is_none());
        }

        {
            proot.create_tracee(Pid::from_raw(0), fs);
        }

        // tracee 0 should exist
        {
            let (tracee, _) = proot.get_mut_tracee_and_all(Pid::from_raw(0));
            assert!(tracee.is_some());
        }
    }
}
