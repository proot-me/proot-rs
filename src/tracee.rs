use nix::sys::ioctl::libc::pid_t;
use nix::sys::signal::Signal;

mod ptrace_events {
    use nix::sys::ptrace::ptrace::*;
    use nix::sys::ioctl::libc::c_int;
    use nix::sys::signal::Signal::{SIGTRAP, SIGSTOP};

    pub type PTraceSignalEvent = c_int;

    pub const RAW_SIGTRAP_SIGNAL:       PTraceSignalEvent = SIGTRAP as c_int;
    pub const NORMAL_SIGTRAP_SIGNAL:    PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | 0x80;
    pub const SECCOMP_SIGNAL:           PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_SECCOMP << 8;
    //TODO: pub const SECCOMP2_SIGNAL:  PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_SECCOMP2 << 8;
    pub const VFORK_SIGNAL:             PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_VFORK << 8;
    pub const VFORK_DONE_SIGNAL:        PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_VFORK_DONE << 8;
    pub const FORK_SIGNAL:              PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_FORK << 8;
    pub const CLONE_SIGNAL:             PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_CLONE << 8;
    pub const EXEC_SIGNAL:              PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_EXEC << 8;
    pub const EXIT_SIGNAL:              PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_EXIT << 8;
    pub const SIGSTOP_SIGNAL:           PTraceSignalEvent = SIGSTOP as c_int;
}

use self::ptrace_events::*;

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

    pub fn handle_event(&mut self, stop_signal: Signal) {
        println!("stopped tracee: {:?}", self);

        let signal: PTraceSignalEvent = stop_signal as PTraceSignalEvent;

        if signal == RAW_SIGTRAP_SIGNAL || signal == NORMAL_SIGTRAP_SIGNAL {
            if signal == RAW_SIGTRAP_SIGNAL {
                print!("raw ");
            }
            println!("sigtrap!");
        } else if signal == SECCOMP_SIGNAL {
            println!("seccomp!");
        } else if signal == VFORK_SIGNAL || signal == FORK_SIGNAL || signal == CLONE_SIGNAL {
            self.new_child(VFORK_SIGNAL);
        } else if signal == EXIT_SIGNAL || signal == EXEC_SIGNAL || signal == VFORK_DONE_SIGNAL {
            println!("signal 0?");
        } else if signal == SIGSTOP_SIGNAL {
            println!("sigstop!");
        }
    }

    fn new_child(&mut self, event: PTraceSignalEvent) {
        println!("new child: {:?}", event);
    }
}