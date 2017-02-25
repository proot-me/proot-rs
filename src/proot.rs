use tracee::{Tracee, FileSystemNameSpace};
use std::collections::HashMap;

// Nix
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::PTRACE_TRACEME;
use nix::sys::ioctl::libc::pid_t;
use nix::sys::ioctl::libc::c_void;
use nix::unistd::{getpid, fork, execvp, ForkResult};
use nix::sys::signal::{kill, SIGSTOP, sigaction};
use nix::errno::Errno;

const NULL: *mut c_void = 0i32 as *mut c_void;

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

        match fork().expect("launch process failed") {
            ForkResult::Parent { child } => {
                println!("parent {}", getpid());

                // we keep track of the tracees's pid
                self.register_alive_tracee(child);
            }
            ForkResult::Child => {
                println!("child {}", getpid());

                // Declare the tracee as ptraceable
                //let status = ptrace(PTRACE_TRACEME, 0, NULL, NULL) as Errno;

                //kill(getpid(), SIGSTOP);

                //if (getenv("PROOT_NO_SECCOMP") == NULL)
                //    (void) enable_syscall_filtering(tracee);

                //execvp("/bin/sh", "");
                //execvp(tracee->exe, argv[0] != NULL ? argv : default_argv);
            }
        }
    }

    pub fn event_loop(&self) {

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