use std::ptr::null_mut;
use libc::{pid_t, user_regs_struct};
use nix::sys::signal::Signal;
use nix::sys::ptrace::ptrace_setoptions;
use nix::{Result, Error};
use nix::sys::ptrace::ptrace::*;
use nix::sys::ptrace::ptrace;
use syscalls::{syscall_enter, syscall_exit};
use proot::InfoBag;
use regs::fetch_regs;

//TODO: remove this when a nix PR will have added them
mod ptrace_events {
    use nix::sys::ptrace::ptrace::*;
    use nix::sys::ioctl::libc::{c_int, SIGTRAP, SIGSTOP};

    pub type PtraceSignalEvent = c_int;

    pub const PTRACE_S_SIGSTOP:           PtraceSignalEvent = SIGSTOP;
    pub const PTRACE_S_RAW_SIGTRAP:       PtraceSignalEvent = SIGTRAP;
    pub const PTRACE_S_NORMAL_SIGTRAP:    PtraceSignalEvent = SIGTRAP | 0x80;
    pub const PTRACE_S_VFORK:             PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_VFORK << 8;
    pub const PTRACE_S_VFORK_DONE:        PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_VFORK_DONE << 8;
    pub const PTRACE_S_FORK:              PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_FORK << 8;
    pub const PTRACE_S_CLONE:             PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_CLONE << 8;
    pub const PTRACE_S_EXEC:              PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_EXEC << 8;
    pub const PTRACE_S_SECCOMP:           PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_SECCOMP << 8;
    pub const PTRACE_S_SECCOMP2:          PtraceSignalEvent = SIGTRAP | (PTRACE_EVENT_SECCOMP + 1) << 8;
    // unreachable pattern?
    // pub const EXIT_SIGNAL:               PTraceSignalEvent = SIGTRAP | PTRACE_EVENT_EXIT << 8;
}
use self::ptrace_events::*;

#[derive(Debug)]
pub enum TraceeStatus {
    /// Enter syscall
    SysEnter,
    /// Exit syscall with no error
    SysExit,
    /// Exit syscall with error
    Error(Error)
}

impl TraceeStatus {
    pub fn is_err(&self) -> bool {
        match *self {
            TraceeStatus::Error(_) => true,
            _ => false
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TraceeRestartMethod {
    /// Restart the tracee, without going through the exit stage
    WithoutExitStage,   // PTRACE_CONT
    /// Restart the tracee, with the exit stage
    WithExitStage,       // PTRACE_SYSCALL,
    /// Do not restart the tracee
    None
}

#[derive(Debug)]
pub struct Tracee {
    /// Process identifier.
    pid: pid_t,
    /// Whether the tracee is in the enter or exit stage
    status: TraceeStatus,
    /// The ptrace's restart method depends on the status (enter or exit) and seccomp on/off
    restart_how: TraceeRestartMethod,
    /// State of the seccomp acceleration for this tracee.
    seccomp: bool,
    /// Ensure the sysexit stage is always hit under seccomp.
    sysexit_pending: bool
}

impl Tracee {
    pub fn new(pid: pid_t) -> Tracee {
        Tracee {
            pid: pid,
            seccomp: false,
            status: TraceeStatus::SysEnter, // it always starts by the enter stage
            restart_how: TraceeRestartMethod::None,
            sysexit_pending: false,
        }
    }

    /// The traced process is stopped; this function will either:
    /// 1. in case of standard syscall: translate the system call's parameters and restart it
    /// 2. in case of fork/clone event: create a new tracee
    /// 3. in other cases: not much
    pub fn handle_event(&mut self, info_bag: &mut InfoBag, stop_signal: Option<Signal>) {
        let signal: PtraceSignalEvent = match stop_signal {
            Some(sig)   => sig as PtraceSignalEvent,
            None        => PTRACE_S_NORMAL_SIGTRAP
        };

        // the restart method might already have been set elsewhere
        if self.restart_how == TraceeRestartMethod::None {
            // When seccomp is enabled, all events are restarted in
            // non-stop mode, but this default choice could be overwritten
            // later if necessary.  The check against "sysexit_pending"
            // ensures WithExitStage/PTRACE_SYSCALL (used to hit the exit stage under
            // seccomp) is not cleared due to an event that would happen
            // before the exit stage, eg. PTRACE_EVENT_EXEC for the exit
            // stage of syscalls.execve(2).
            if self.seccomp && !self.sysexit_pending {
                self.restart_how = TraceeRestartMethod::WithoutExitStage;
            } else {
                self.restart_how = TraceeRestartMethod::WithExitStage;
            }
        }

        match signal {
            PTRACE_S_RAW_SIGTRAP| PTRACE_S_NORMAL_SIGTRAP   => self.handle_sigtrap_event(info_bag, signal),
            PTRACE_S_SECCOMP | PTRACE_S_SECCOMP2            => self.handle_seccomp_event(info_bag, signal),
            PTRACE_S_VFORK | PTRACE_S_FORK | PTRACE_S_CLONE => self.new_child(signal),
            PTRACE_S_EXEC | PTRACE_S_VFORK_DONE             => self.handle_exec_vfork_event(),
            PTRACE_S_SIGSTOP                                => self.handle_sigstop_event(),
            _ => {}
        }
    }

    fn handle_sigstop_event(&mut self) {
        println!("sigstop! {}", self.pid);

        // Stop this tracee until PRoot has received
        // the EVENT_*FORK|CLONE notification.
        //if (tracee->exe == NULL) {
        //    tracee->sigstop = SIGSTOP_PENDING;
        //    self.restart_how = TraceeRestartMethod::None;
        //    return TraceeRestartSignal::Stopped;
        //}
    }

    fn handle_exec_vfork_event(&mut self) {
        println!("EXEC or VFORK event");
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
                TraceeStatus::SysExit | TraceeStatus::Error(_)  => {
                    // sysexit: the next sysenter will be notified by seccomp.
                    self.restart_how = TraceeRestartMethod::WithoutExitStage;
                    self.sysexit_pending = false;
                }
            }
        }
        self.translate_syscall();
    }

    /// Retrieves the registers,
    /// handles either the enter or exit stage of the system call,
    /// and pushes the registers.
    fn translate_syscall(&mut self) {
        // We retrieve the registers of the current tracee.
        // They contain the system call's number, arguments and other register's info.
        let regs = match fetch_regs(self.pid) {
            Ok(regs) => regs,
            Err(_)  => return
        };

        match self.status {
            TraceeStatus::SysEnter => {
                /* Never restore original register values at the end
                 * of this stage.  */
                // tracee->restore_original_regs = false;

                // save_current_regs(tracee, ORIGINAL);
                let status = self.translate_syscall_enter(&regs);
                // save_current_regs(tracee, MODIFIED);

                if status.is_err() {
                    // Remember the tracee status for the "exit" stage and
                    // avoid the actual syscall if an error was reported
                    // by the translation/extension.
                    // set_sysnum(tracee, PR_void);
                    // poke_reg(tracee, SYSARG_RESULT, (word_t) status);
                    self.status = TraceeStatus::Error(status.unwrap_err());
                } else {
                    self.status = TraceeStatus::SysExit;
                }

                // Restore tracee's stack pointer now if it won't hit
                // the sysexit stage (i.e. when seccomp is enabled and
                // there's nothing else to do).
                if self.restart_how == TraceeRestartMethod::WithoutExitStage  {
                    self.status = TraceeStatus::SysEnter;
                    // poke_reg(tracee, STACK_POINTER, peek_reg(tracee, ORIGINAL, STACK_POINTER));
                }
            }
            TraceeStatus::SysExit | TraceeStatus::Error(_) => {
                /* By default, restore original register values at the
                 * end of this stage.  */
                // tracee->restore_original_regs = true;

                self.translate_syscall_exit(&regs);

                // reset the tracee's status
                self.status = TraceeStatus::SysEnter;
            }
        }

        // push_regs
    }

    fn translate_syscall_enter(&mut self, regs: &user_regs_struct) -> Result<()> {
        // status = notify_extensions(tracee, SYSCALL_ENTER_START, 0, 0);
        // if (status < 0)
        //     goto end;
        // if (status > 0)
        //     return 0;

        let status = syscall_enter::translate(self.pid, regs);

        // status2 = notify_extensions(tracee, SYSCALL_ENTER_END, status, 0);
        // if (status2 < 0)
        //     status = status2;

        status
    }

    fn translate_syscall_exit(&mut self, regs: &user_regs_struct) {
        // status = notify_extensions(tracee, SYSCALL_EXIT_START, 0, 0);
        // if (status < 0) {
        //     poke_reg(tracee, SYSARG_RESULT, (word_t) status);
        //     goto end;
        // }
        // if (status > 0)
        //     return;

        // Set the tracee's errno if an error occured previously during the translation.
        if self.status.is_err() {
            // poke_reg(tracee, SYSARG_RESULT, (word_t) tracee->status);
        } else {
            let syscall_exit_result = syscall_exit::translate(regs);

            if !syscall_exit_result.is_none() {
                // poke_reg(tracee, SYSARG_RESULT, (word_t) status.get_value());
            }
        }

        // status = notify_extensions(tracee, SYSCALL_EXIT_END, 0, 0);
        // if (status < 0)
        //     poke_reg(tracee, SYSARG_RESULT, (word_t) status);
    }

    fn handle_seccomp_event(&mut self, info_bag: &mut InfoBag, signal: PtraceSignalEvent) {
        println!("seccomp event! {:?}, {:?}", info_bag, signal);
    }

    fn new_child(&mut self, event: PtraceSignalEvent) {
        println!("new child: {:?}", event);
    }

    pub fn restart(&mut self) {
        match self.restart_how {
            TraceeRestartMethod::WithoutExitStage => {
                ptrace(PTRACE_CONT, self.pid, null_mut(), null_mut()).expect("exit tracee without exit stage");
            },
            TraceeRestartMethod::WithExitStage => {
                ptrace(PTRACE_SYSCALL, self.pid, null_mut(), null_mut()).expect("exit tracee with exit stage");
            },
            TraceeRestartMethod::None => {}
        };

        // the restart method is reinitialised here
        self.restart_how = TraceeRestartMethod::None;
    }


    /// Distinguish some events from others and
    /// automatically trace each new process with
    /// the same options.
    ///
    /// Note that only the first bare SIGTRAP is
    /// related to the tracing loop, others SIGTRAP
    /// carry tracing information because of
    /// TRACE*FORK/CLONE/EXEC.
    pub fn set_ptrace_options(&self, info_bag: &mut InfoBag) {
        if info_bag.deliver_sigtrap {
            return;
        } else {
            info_bag.deliver_sigtrap = true;
        }

        let default_options =
            PTRACE_O_TRACESYSGOOD |
                PTRACE_O_TRACEFORK |
                PTRACE_O_TRACEVFORK |
                PTRACE_O_TRACEVFORKDONE |
                PTRACE_O_TRACEEXEC |
                PTRACE_O_TRACECLONE |
                PTRACE_O_TRACEEXIT;

        //TODO: seccomp
        ptrace_setoptions(self.pid, default_options).expect("set ptrace options");
    }

    #[cfg(test)]
    pub fn get_pid(&self) -> pid_t { self.pid }
}


#[cfg(test)]
mod tests {
    use super::*;
    use utils::tests::fork_test;

    #[test]
    fn create_tracee() {
        let tracee = Tracee::new(42);
        assert_eq!(tracee.get_pid(), 42);
    }

    #[test]
    /// This test tests that the set_ptrace_options runs without panicking.
    /// It requires a traced child process to be applied on,
    /// as using `ptrace(PTRACE_SETOPTIONS)` without preparation results in a Sys(ESRCH) error.
    fn create_set_ptrace_options() {
        fork_test(
            // expecting a normal execution
            0,
            // parent
            |_, _| {
                // we stop on the first syscall;
                // the fact that no panic was sparked until now
                // means that the set_trace_options call was OK
                return true;
            },
            // child
            || {});
    }
}