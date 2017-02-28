use tracee::Tracee;
use fsnamespace::FileSystemNameSpace;
use std::collections::HashMap;
use std::ptr::null_mut;
use std::ffi::CString;

// libc
use nix::sys::ioctl::libc::{pid_t, siginfo_t, c_int, c_void};
// ptrace
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::PTRACE_TRACEME;
// fork
use nix::unistd::{getpid, fork, execvp, ForkResult};
// signals
use nix::sys::signal::{kill, sigaction, Signal, SigAction, SigSet, SigHandler};
use nix::sys::signal::{SaFlags, SA_SIGINFO, SA_RESTART};
use nix::sys::signal::Signal::*;
// event loop
use nix::sys::wait::{waitpid, __WALL};
use nix::sys::wait::WaitStatus::*;


#[derive(Debug)]
pub struct PRoot {
    main_pid: pid_t,
    tracees: HashMap<pid_t, Tracee>,
    alive_tracees: Vec<pid_t>,
    /// Information related to a file-system name-space.
    fs: FileSystemNameSpace
}

impl PRoot {
    pub fn new(fs: FileSystemNameSpace) -> PRoot {
        PRoot {
            main_pid: getpid(),
            tracees: HashMap::new(),
            alive_tracees: vec![],
            fs: fs
        }
    }

    /// Main process where proot splits into two threads:
    /// - a tracer, the parent thread.
    /// - a (first) tracee, the child thread,
    ///   that will declare itself as ptrace-able before executing the program.
    ///
    /// Attention: `fork()` implies that the OS will apply copy-on-write
    /// on all the shared memory of the parent and child processes
    /// (heap, libraries...), so both of them will have their own (owned) version
    /// of the PRoot memory (so the `fs`` field mainly)
    pub fn launch_process(&mut self) {

        match fork().expect("launch process") {
            ForkResult::Parent { child } => {
                // we create the first tracee
                self.create_tracee(child);
            }
            ForkResult::Child => {
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

    /// Configures the action associated with specific signals.
    /// All signals are blocked when the signal handler is called.
    /// SIGINFO is used to know which process has signaled us and
    /// RESTART is used to restart waitpid(2) seamlessly.
    pub fn prepare_sigactions(&self) {
        let signal_set: SigSet = SigSet::all();
        let sa_flags: SaFlags = SA_SIGINFO | SA_RESTART;

        for signal in Signal::iterator() {
            let mut signal_handler: SigHandler = SigHandler::SigIgn; // default action is ignoring

            // setting the action when receiving certain signals
            match signal {
                SIGQUIT | SIGILL | SIGABRT | SIGFPE | SIGSEGV => {
                    // tracees on abnormal termination signals
                    signal_handler = SigHandler::SigAction(stop_program);
                }
                SIGUSR1 | SIGUSR2 => {
                    // can be used for inter-process communication
                    signal_handler = SigHandler::Handler(show_info);
                }
                SIGCHLD | SIGCONT | SIGTSTP | SIGTTIN | SIGTTOU => {
                    // these signals are related to tty and job control,
                    // so we keep the default action for them
                    continue;
                }
                SIGSTOP | SIGKILL => {
                    // these two signals cannot be used with sigaction
                    continue;
                }
                _ => {} // all other signals (even ^C) are ignored
            }

            let signal_action = SigAction::new(signal_handler, sa_flags, signal_set);
            unsafe {
                match sigaction(signal, &signal_action) {
                    Err(err) => {
                        println!("Warning: sigaction failed for signal {:?} : {:?}.", signal, err);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Infinite loop where PRoot will wait for tracees signals with `waitpid`.
    /// Tracees are stopped when
    pub fn event_loop(&mut self) {
        loop {
            // free_terminated_tracees();

            match waitpid(-1, Some(__WALL)).expect("event loop waitpid") {
                Exited(pid, exit_status) => {
                    println!("{}, Exited with status: {}", pid, exit_status);
                    self.register_tracee_finished(pid);
                }
                Signaled(pid, term_signal, dumped_core) => {
                    println!("{}, Signaled with status: {:?}, and dump core: {}", pid, term_signal, dumped_core);
                    self.register_tracee_finished(pid);
                }
                Stopped(pid, stop_signal) => {
                    println!("{}, Stopped", pid);
                    let mut tracee = self.get_mut_tracee(pid).expect("get stopped tracee");

                    tracee.handle_event(stop_signal);
                }
                _ => {}
            }
        }
    }

    /******** Utilities ****************/

    pub fn is_main_thread(&self) -> bool { getpid() == self.main_pid }

    pub fn create_tracee(&mut self, pid: pid_t) -> Option<&Tracee> {
        self.tracees.insert(pid, Tracee::new(pid));
        self.register_alive_tracee(pid);
        self.tracees.get(&pid)
    }

    pub fn get_tracee(&self, pid: pid_t) -> Option<&Tracee> { self.tracees.get(&pid)  }
    fn get_mut_tracee(&mut self, pid: pid_t) -> Option<&mut Tracee> { self.tracees.get_mut(&pid) }

    fn register_alive_tracee(&mut self, pid: pid_t) {
        self.alive_tracees.push(pid);
    }

    fn register_tracee_finished(&mut self, finished_pid: pid_t) {
        self.alive_tracees.retain(|pid| *pid != finished_pid);
        self.tracees.remove(&finished_pid);
    }
}


/// Proot has received a fatal error from one of the tracee,
/// and must therefore stop the program's execution.
extern "C" fn stop_program(sig_num: c_int, _: *mut siginfo_t, _: *mut c_void) {
    let signal = Signal::from_c_int(sig_num).unwrap();
    panic!("abnormal signal received: {:?}", signal);
}

extern "C" fn show_info(pid: pid_t) {
    println!("showing info pid {}", pid);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_tracee_test() {
        let fs = FileSystemNameSpace::new();
        let mut proot = PRoot::new(fs);

        // tracee 0 shouldn't exist
        assert!(proot.get_tracee(0).is_none());

        { proot.create_tracee(0); }

        // tracee 0 should exist
        assert!(proot.get_tracee(0).is_some());
    }

    #[test]
    fn prepare_sigactions_test() {
        let fs = FileSystemNameSpace::new();
        let proot = PRoot::new(fs);

        // should pass without panicking
        proot.prepare_sigactions();
    }
}