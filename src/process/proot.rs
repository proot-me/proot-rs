use std::ffi::CString;
use std::process;
use std::{collections::HashMap, convert::TryFrom};

use libc::{c_int, c_void, pid_t, siginfo_t};
use nix::sys::ptrace::{self, Event as PtraceEvent};
use nix::sys::signal::{self, Signal};
use nix::sys::wait::{self, WaitPidFlag, WaitStatus::*};
use nix::unistd::{self, ForkResult, Pid};

use crate::process::event::EventHandler;
use crate::process::tracee::Tracee;
use crate::{
    errors::{Result, WithContext},
    filesystem::{temp::TempFile, FileSystem},
};

/// Used to store global info common to all tracees. Rename into
/// `Configuration`?
#[derive(Debug)]
pub struct InfoBag {
    /// Used to know if the ptrace options is already set
    pub options_already_set: bool,
    /// Binary loader, used by `execve`.
    /// The content of the binary is actually inlined in `proot-rs`
    /// (see `src/kernel/execve/loader`), and is extracted into a temporary file
    /// before use. This temporary file struct makes sure the file is
    /// deleted when it's dropped.
    pub loader: TempFile,
}

impl InfoBag {
    pub fn new() -> InfoBag {
        InfoBag {
            options_already_set: false,
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
    /// - a (first) tracee, the child thread, that will declare itself as
    ///   ptrace-able before executing the program.
    ///
    /// The `fork()` done here implies that the OS will apply copy-on-write
    /// on all the shared memory of the parent and child processes
    /// (heap, libraries...), so both of them will have their own (owned)
    /// version of the PRoot memory.
    pub fn launch_process(&mut self, initial_fs: FileSystem, command: Vec<String>) -> Result<()> {
        // parse command
        assert!(command.len() > 0); // TODO: remove this
        let args = command
            .iter()
            .map(|arg| {
                CString::new(arg.as_bytes()).with_context(|| {
                    format!("Illegal parameters, should not contain \0 bytes: {}", arg)
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let filename = &args[0];
        match unsafe { unistd::fork() }.context("Failed to fork() when starting process")? {
            ForkResult::Parent { child } => {
                // we create the first tracee
                self.create_tracee(child, initial_fs);
            }
            ForkResult::Child => {
                let init_child_func = || -> Result<()> {
                    // Declare the tracee as ptraceable
                    ptrace::traceme()
                        .context("Failed to execute ptrace::traceme() in a child process")?;
                    // Synchronise with the parent's event loop by waiting until it's ready
                    // (otherwise the execvp is executed too quickly)
                    signal::kill(unistd::getpid(), Signal::SIGSTOP)
                        .context("Child process failed to synchronize with parent process")?;
                    //TODO: seccomp
                    //if (getenv("PROOT_NO_SECCOMP") == NULL)
                    //    (void) enable_syscall_filtering(tracee);
                    unistd::execvp(&filename, &args).with_context(|| {
                        format!("Failed to call execvp() with command: {:?}", command)
                    })?;
                    unreachable!()
                };

                if let Err(e) = init_child_func() {
                    error!("Failed to initialize the child process: {}", e);
                    // Ensure that child processes will not return to the main function
                    process::exit(1);
                }
            }
        };
        Ok(())
    }

    /// Infinite loop where PRoot will wait for tracees signals with `waitpid`.
    /// Tracees will be stopped when they use a system call.
    /// The tracer will be notified through `waitpid` and will be able to alter
    /// the parameters of the system call, before restarting the tracee.
    pub fn event_loop(&mut self) -> Result<()> {
        // TODO: what should we do if there is a terrible error in eventloop?
        while !self.alive_tracees.is_empty() {
            match wait::waitpid(Pid::from_raw(-1), Some(WaitPidFlag::__WALL))
                .context("Error calling waitpid() in event loop")?
            {
                Exited(pid, exit_status) => {
                    trace!("-- {}, Exited with status: {}", pid, exit_status);
                    self.register_tracee_finished(pid);
                }
                Signaled(pid, term_signal, dumped_core) => {
                    trace!(
                        "-- {}, Signaled with status: {:?}, and dump core: {}",
                        pid,
                        term_signal,
                        dumped_core
                    );
                    self.register_tracee_finished(pid);
                }
                // The tracee was stopped by a normal signal (signal-delivery-stop), or was stopped
                // by a system call (syscall-stop) with PTRACE_O_TRACESYSGOOD not effect.
                Stopped(pid, stop_signal) => {
                    trace!(
                        "-- {}, Stopped, {:?}, {}",
                        pid,
                        stop_signal,
                        stop_signal as c_int
                    );
                    let tracee = self.tracees.get_mut(&pid).expect("get stopped tracee");
                    tracee.reset_restart_how();
                    match stop_signal {
                        Signal::SIGSTOP => {
                            // When the first child process starts, it sends a SIGSTOP to itself.
                            // And we need to set ptrace options at this point.
                            tracee.check_and_set_ptrace_options(&mut self.info_bag)?;

                            tracee.handle_sigstop_event()
                        }
                        Signal::SIGTRAP => {
                            // Since PTRACE_O_TRACESYSGOOD is not supported on older versions of
                            // Linux (version<2.4.6) and some architectures, we need to use
                            // PTRACE_GETSIGINFO to distinguish a real syscall-stop from
                            // signal-delivery-stop on these devices.
                            // NOTE: this may be somewhat expensive.
                            // See ptrace(2): Syscall-stops
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

                    // ptrace(2): If the tracer doesn't suppress the signal, it passes the signal to
                    // the tracee in the next ptrace restart request.
                    // TODO: we should deliver this signal(sig) with ptrace(PTRACE_restart, pid, 0,
                    // sig)
                    tracee.restart(Some(stop_signal));
                }
                // The tracee was stopped by a SIGTRAP with additional status (PTRACE_EVENT stops).
                PtraceEvent(pid, signal, status_additional) => {
                    trace!(
                        "-- {}, Ptrace event, {:?}, {:?}",
                        pid,
                        signal,
                        status_additional
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
                    tracee.restart(None);
                }
                // The tracee was stopped by execution of a system call (syscall-stop), and
                // PTRACE_O_TRACESYSGOOD was effect. PTRACE_O_TRACESYSGOOD is used to make it
                // easy for the tracer to distinguish syscall-stop from signal-delivery-stop.
                PtraceSyscall(pid) => {
                    trace!("-- {}, Syscall", pid);
                    let tracee = self.tracees.get_mut(&pid).expect("get stopped tracee");
                    tracee.reset_restart_how();
                    tracee.handle_syscall_stop_event(&mut self.info_bag);
                    tracee.restart(None);
                }
                Continued(pid) => {
                    trace!("-- {}, Continued", pid);
                }
                StillAlive => {
                    trace!("-- Still alive");
                }
            }
        }

        Ok(())
    }

    /******** Utilities *************** */

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
