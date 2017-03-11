use nix::sys::ioctl::libc::pid_t;
use nix::sys::signal::Signal;
use nix::sys::ptrace::ptrace_setoptions;
use proot::InfoBag;
use nix::sys::ptrace::ptrace::*;
use constants::ptrace::ptrace_events::*;

#[derive(Debug)]
pub struct Tracee {
    /// Process identifier.
    pid: pid_t
}

impl Tracee {
    pub fn new(pid: pid_t) -> Tracee {
        Tracee {
            pid: pid
        }
    }

    pub fn handle_event(&mut self, info_bag: &mut InfoBag, stop_signal: Signal) {
        println!("stopped tracee: {:?}", self);

        let signal: PTraceSignalEvent = stop_signal as PTraceSignalEvent;

        match signal {
            RAW_SIGTRAP_SIGNAL | NORMAL_SIGTRAP_SIGNAL => {
                // it's either the first SIGTRAP signal, or a standard system call
                if signal == RAW_SIGTRAP_SIGNAL {
                    self.set_ptrace_options(info_bag);
                }

                self.translate_syscall();
            }
            SECCOMP_SIGNAL => {
                println!("seccomp!");
            }
            VFORK_SIGNAL | FORK_SIGNAL | CLONE_SIGNAL => {
                self.new_child(signal);
            }
            EXEC_SIGNAL | VFORK_DONE_SIGNAL => {
                println!("signal 0?");
            }
            SIGSTOP_SIGNAL => {
                println!("sigstop! {}", self.pid);
            }
            _ => {}
        }
    }

    fn translate_syscall(&mut self) {}

    /// Distinguish some events from others and
    /// automatically trace each new process with
    /// the same options.
    /// Note that only the first bare SIGTRAP is
    /// related to the tracing loop, others SIGTRAP
    /// carry tracing information because of
    /// TRACE*FORK/CLONE/EXEC.
    fn set_ptrace_options(&self, info_bag: &mut InfoBag) {
        let default_options =
            PTRACE_O_TRACESYSGOOD |
                PTRACE_O_TRACEFORK |
                PTRACE_O_TRACEVFORK |
                PTRACE_O_TRACEVFORKDONE |
                PTRACE_O_TRACEEXEC |
                PTRACE_O_TRACECLONE |
                PTRACE_O_TRACEEXIT;

        if info_bag.deliver_sigtrap {
            return;
        } else {
            info_bag.deliver_sigtrap = true;
        }

        //TODO: seccomp
        ptrace_setoptions(self.pid, default_options).expect("set ptrace options");
    }

    fn new_child(&mut self, event: PTraceSignalEvent) {
        println!("new child: {:?}", event);
    }

    #[cfg(test)]
    pub fn get_pid(&self) -> pid_t { self.pid }
}


#[cfg(test)]
mod tests {
    use super::*;
    use proot::InfoBag;
    use std::ptr::null_mut;
    use nix::unistd::{getpid, fork, ForkResult};
    use nix::sys::signal::{kill};
    use nix::sys::signal::Signal::{SIGSTOP};
    use nix::sys::ptrace::ptrace;
    use nix::sys::ptrace::ptrace::PTRACE_TRACEME;
    use nix::sys::wait::{waitpid, __WALL};
    use nix::sys::wait::WaitStatus::*;

    #[test]
    fn create_tracee() {
        let tracee = Tracee::new(42);
        assert_eq!(tracee.get_pid(), 42);
    }

    #[test]
    /// This test tests that the set_ptrace_options runs without panicking.
    /// It requires a traced child process to be apply on,
    /// as using `ptrace(PTRACE_SETOPTIONS)` without preparation results in a Sys(ESRCH) error.
    fn create_set_ptrace_options() {
        match fork().expect("fork in set ptrace options tracee's test") {
            ForkResult::Parent { child } => {
                let info_bag = &mut InfoBag::new();
                let tracee = Tracee::new(child);

                assert_eq!(info_bag.deliver_sigtrap, false);

                // The parent will wait for the child's signal before calling set_ptrace_options
                match waitpid(-1, Some(__WALL)).expect("event loop waitpid") {
                    Stopped(_, stop_signal) => {
                        assert_eq!(stop_signal, SIGSTOP);
                    }
                    _ => {
                        assert!(false);
                    }
                }
                // This call must pass without panic
                tracee.set_ptrace_options(info_bag);

                // if everything went right, this boolean should have become true
                assert_eq!(info_bag.deliver_sigtrap, true);
            }
            ForkResult::Child => {
                ptrace(PTRACE_TRACEME, 0, null_mut(), null_mut()).expect("test ptrace traceme");
                // we use a SIGSTOP to synchronise both processes
                kill(getpid(), SIGSTOP).expect("test child sigstop");
            }
        }

    }
}