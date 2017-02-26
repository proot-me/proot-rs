use tracee::{Tracee, FileSystemNameSpace};
use std::collections::HashMap;
use std::ptr::null_mut;
use std::ffi::CString;

// Nix
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::PTRACE_TRACEME;
use nix::sys::ioctl::libc::pid_t;
use nix::unistd::{getpid, fork, execvp, ForkResult};
use nix::sys::signal::{kill, sigaction, Signal, SigAction, SigSet, SigHandler};
use nix::sys::signal::{SaFlags, SA_SIGINFO, SA_RESTART};
use nix::sys::signal::Signal::*; // all 31 signals

#[derive(Debug)]
pub struct PRoot {
    main_pid: pid_t,
    tracees: HashMap<pid_t, Tracee>,
    alive_tracees: Vec<pid_t>
}

impl PRoot {
    pub fn new() -> PRoot {
        PRoot {
            main_pid: getpid(),
            tracees: HashMap::new(),
            alive_tracees: vec![]
        }
    }

    /// Main process where proot splits into two threads:
    /// - a tracer, the parent thread.
    /// - a (first) tracee, the child thread,
    ///   that will declare itself as ptrace-able before executing the program.
    ///
    /// Attention: `fork()` implies that the OS will apply copy-on-write
    /// on all the shared memory of the parent and child processes
    /// (heap, libraries...), so both of them will have their own version
    /// of the PRoot memory.
    pub fn launch_process(&mut self) {

        match fork().expect("launch process") {
            ForkResult::Parent { child } => {
                println!("parent {}", getpid());

                // we keep track of the tracees's pid
                self.register_alive_tracee(child);


            }
            ForkResult::Child => {
                println!("child {}", getpid());

                // Declare the tracee as ptraceable
                ptrace(PTRACE_TRACEME, 0, null_mut(), null_mut()).expect("ptrace traceme");

                // Synchronise with the parent's event loop by waiting until it's ready
                // (otherwise the execvp is executed too quickly)
                kill(getpid(), SIGSTOP).expect("first child synchronisation");

                //if (getenv("PROOT_NO_SECCOMP") == NULL)
                //    (void) enable_syscall_filtering(tracee);

                execvp(&CString::new("echo").unwrap(), &[CString::new(".").unwrap(), CString::new("TEST").unwrap()])
                    .expect("failed execvp");
                //execvp(tracee->exe, argv[0] != NULL ? argv : default_argv);
            }
        }
    }

    pub fn event_loop(&self) {
        let signal_set: SigSet = SigSet::all();
        // all signal are blocked when the signal handler is called;
        // SIGINFO is used to know which process has signaled us and
        // RESTART is used to restart waitpid(2) seamlessly
        let sa_flags: SaFlags = SA_SIGINFO | SA_RESTART;

        for signal in Signal::iterator() {
            let mut signal_handler: SigHandler = SigHandler::SigIgn; // default action is ignoring
            let signal_action: SigAction;

            // setting the action when receiving certain signals
            match signal {
                SIGQUIT | SIGILL | SIGABRT | SIGFPE | SIGSEGV => {
                    // tracees on abnormal termination signals
                    signal_handler = SigHandler::Handler(kill_tracee);
                }
                SIGUSR1 | SIGUSR2 => {
                    // can be used for inter-process communication
                    signal_handler = SigHandler::Handler(show_info);
                }
                SIGCHLD | SIGCONT | SIGSTOP | SIGTSTP | SIGTTIN | SIGTTOU => {
                    // these signals are related to tty and job control,
                    // so we keep the default action for them
                    continue;
                }
                _ => {
                    // all other signals (even ^C) are ignored
                }
            }

            signal_action = SigAction::new(signal_handler, sa_flags, signal_set);
            unsafe {
                match sigaction(signal, &signal_action) {
                    Err(err) => {
                        println!("Warning: sigaction failed for signal {:?} : {:?}.", signal, err);
                    }
                    _ => ()
                }
            }
        }
    }

    /******** Utilities ****************/

    pub fn is_main_thread(&self) -> bool { getpid() == self.main_pid }

    pub fn create_tracee(&mut self, pid: pid_t, fs: FileSystemNameSpace) -> Option<&Tracee> {
        self.tracees.insert(pid, Tracee::new(pid, fs));
        self.tracees.get(&pid)
    }

    /// For read-only operations
    pub fn get_tracee(&self, pid: pid_t) -> Option<&Tracee> { self.tracees.get(&pid)  }

    /// For read-only operations
    //pub fn get_mut_tracee(&mut self, pid: pid_t) -> Option<&mut Tracee> {
    //    self.tracees.get_mut(&pid)
    //}

    fn register_alive_tracee(&mut self, pid: pid_t) {
        self.alive_tracees.push(pid);
    }
}


extern "C" fn kill_tracee(pid: pid_t) {
    println!("killing pid {}", pid);
}

extern "C" fn show_info(pid: pid_t) {
    println!("showing info pid {}", pid);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_tracee() {
        let mut proot = PRoot::new();
        let fs = FileSystemNameSpace::new();

        // tracee 0 shouldn't exist
        assert!(proot.get_tracee(0).is_none());

        { proot.create_tracee(0, fs); }

        // tracee 0 should exist
        assert!(proot.get_tracee(0).is_some());
    }
}