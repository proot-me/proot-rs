use crate::process::proot::InfoBag;
use crate::process::tracee::{Tracee, TraceeRestartMethod, TraceeStatus};
use crate::process::translation::SyscallTranslator;
use nix::sys::signal::Signal;

//TODO: remove this when a nix PR will have added them
mod ptrace_events {
    use libc::{c_int, SIGSTOP, SIGTRAP};
    use nix::sys::ptrace::ptrace::*;

    pub type PtraceSignalEvent = c_int;

    pub const PTRACE_S_SIGSTOP: PtraceSignalEvent = SIGSTOP;
    pub const PTRACE_S_RAW_SIGTRAP: PtraceSignalEvent = SIGTRAP;
    pub const PTRACE_S_NORMAL_SIGTRAP: PtraceSignalEvent = SIGTRAP | 0x80;
    pub const PTRACE_S_VFORK: PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_VFORK << 8;
    pub const PTRACE_S_VFORK_DONE: PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_VFORK_DONE << 8;
    pub const PTRACE_S_FORK: PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_FORK << 8;
    pub const PTRACE_S_CLONE: PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_CLONE << 8;
    pub const PTRACE_S_EXEC: PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_EXEC << 8;
    pub const PTRACE_S_SECCOMP: PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_SECCOMP << 8;
    pub const PTRACE_S_SECCOMP2: PtraceSignalEvent = SIGTRAP | (PTRACE_EVENT_SECCOMP + 1) << 8;
    // unreachable pattern?
    // pub const EXIT_SIGNAL:               PTraceSignalEvent = SIGTRAP | PTRACE_EVENT_EXIT << 8;
}
use self::ptrace_events::*;

pub trait EventHandler {
    fn handle_event(&mut self, info_bag: &mut InfoBag, stop_signal: Option<Signal>);
    fn handle_sigtrap_event(&mut self, info_bag: &mut InfoBag, signal: PtraceSignalEvent);
    fn handle_sigstop_event(&mut self);
    fn handle_seccomp_event(&mut self, info_bag: &mut InfoBag, signal: PtraceSignalEvent);
    fn handle_exec_vfork_event(&mut self);
    fn handle_new_child_event(&mut self, event: PtraceSignalEvent);
}

impl EventHandler for Tracee {
    /// The traced process is stopped; this function will either:
    /// 1. in case of standard syscall: translate the system call's parameters and restart it
    /// 2. in case of fork/clone event: create a new tracee
    /// 3. in other cases: not much
    fn handle_event(&mut self, info_bag: &mut InfoBag, stop_signal: Option<Signal>) {
        let signal: PtraceSignalEvent = match stop_signal {
            Some(sig) => sig as PtraceSignalEvent,
            None => PTRACE_S_NORMAL_SIGTRAP,
        };

        // the restart method might already have been set elsewhere
        if self.restart_how == TraceeRestartMethod::None {
            // When seccomp is enabled, all events are restarted in
            // non-stop mode, but this default choice could be overwritten
            // later if necessary.  The check against "sysexit_pending"
            // ensures WithExitStage/PTRACE_SYSCALL (used to hit the exit stage under
            // seccomp) is not cleared due to an event that would happen
            // before the exit stage, eg. PTRACE_EVENT_EXEC for the exit
            // stage of kernel.execve(2).
            if self.seccomp && !self.sysexit_pending {
                self.restart_how = TraceeRestartMethod::WithoutExitStage;
            } else {
                self.restart_how = TraceeRestartMethod::WithExitStage;
            }
        }

        match signal {
            PTRACE_S_RAW_SIGTRAP | PTRACE_S_NORMAL_SIGTRAP => {
                self.handle_sigtrap_event(info_bag, signal)
            }
            PTRACE_S_SECCOMP | PTRACE_S_SECCOMP2 => self.handle_seccomp_event(info_bag, signal),
            PTRACE_S_VFORK | PTRACE_S_FORK | PTRACE_S_CLONE => self.handle_new_child_event(signal),
            PTRACE_S_EXEC | PTRACE_S_VFORK_DONE => self.handle_exec_vfork_event(),
            PTRACE_S_SIGSTOP => self.handle_sigstop_event(),
            _ => {}
        }
    }

    /// Standard handling of either:
    /// 1. the initial SIGTRAP signal
    /// 2. a syscall that is then translated
    fn handle_sigtrap_event(&mut self, info_bag: &mut InfoBag, signal: PtraceSignalEvent) {
        if signal == PTRACE_S_RAW_SIGTRAP {
            // it's the initial SIGTRAP signal
            self.set_ptrace_options(info_bag);
        }

        /* This tracee got signaled then freed during the
        sysenter stage but the kernel reports the sysexit
        stage; just discard this spurious tracee/event. */
        // if (tracee->exe == NULL) {
        //    self.restart_how = Some(TraceeRestartMethod::WithoutExitStage);
        //    return TraceeRestartSignal::Signal(0);
        // }

        if self.seccomp {
            match self.status {
                TraceeStatus::SysEnter => {
                    // sysenter: ensure the sysexit stage will be hit under seccomp.
                    self.restart_how = TraceeRestartMethod::WithExitStage;
                    self.sysexit_pending = true;
                }
                TraceeStatus::SysExit | TraceeStatus::Error(_) => {
                    // sysexit: the next sysenter will be notified by seccomp.
                    self.restart_how = TraceeRestartMethod::WithoutExitStage;
                    self.sysexit_pending = false;
                }
            }
        }
        self.translate_syscall(info_bag);
    }

    fn handle_sigstop_event(&mut self) {
        println!("sigstop! {}", self.pid);

        // Stop this tracee until PRoot has received
        // the EVENT_*FORK|CLONE notification.
        // if self.exe.is_none() {
        //     tracee->sigstop = SIGSTOP_PENDING;
        //     self.restart_how = TraceeRestartMethod::None;
        //     return TraceeRestartSignal::Stopped;
        // }
    }

    fn handle_seccomp_event(&mut self, info_bag: &mut InfoBag, signal: PtraceSignalEvent) {
        println!("seccomp event! {:?}, {:?}", info_bag, signal);
    }

    fn handle_exec_vfork_event(&mut self) {
        println!("EXEC or VFORK event");
    }

    fn handle_new_child_event(&mut self, event: PtraceSignalEvent) {
        println!("new child: {:?}", event);
    }
}
