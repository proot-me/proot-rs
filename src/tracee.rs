use std::ptr::null_mut;
use nix::sys::ioctl::libc::pid_t;
use nix::sys::signal::Signal;
use nix::sys::ptrace::ptrace_setoptions;
use proot::InfoBag;
use nix::sys::ptrace::ptrace::*;
use nix::sys::ptrace::ptrace;
use constants::ptrace::ptrace_events::*;
use constants::tracee::{TraceeStatus, TraceeRestartMethod};
use regs::fetch_regs;
use regs::regs_structs::user_regs_struct;

#[derive(Debug)]
pub struct Tracee {
    /// Process identifier.
    pid: pid_t,
    /// Whether the tracee is in the enter or exit stage
    status: TraceeStatus,
    /// The ptrace's restart method depends on the status (enter or exit) and seccomp on/off
    restart_how: Option<TraceeRestartMethod>,
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
            restart_how: None,
            sysexit_pending: false
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
        if self.restart_how.is_none() {
            // When seccomp is enabled, all events are restarted in
            // non-stop mode, but this default choice could be overwritten
            // later if necessary.  The check against "sysexit_pending"
            // ensures WithExitStage/PTRACE_SYSCALL (used to hit the exit stage under
            // seccomp) is not cleared due to an event that would happen
            // before the exit stage, eg. PTRACE_EVENT_EXEC for the exit
            // stage of execve(2).
            if self.seccomp && !self.sysexit_pending {
                self.restart_how = Some(TraceeRestartMethod::WithoutExitStage);
            } else {
                self.restart_how = Some(TraceeRestartMethod::WithExitStage);
            }
        }

        match signal {
            PTRACE_S_RAW_SIGTRAP| PTRACE_S_NORMAL_SIGTRAP => self.handle_sigtrap_event(info_bag, signal),
            PTRACE_S_SECCOMP | PTRACE_S_SECCOMP2 => self.handle_seccomp_event(info_bag, signal),
            PTRACE_S_VFORK | PTRACE_S_FORK | PTRACE_S_CLONE => self.new_child(signal),
            PTRACE_S_EXEC | PTRACE_S_VFORK_DONE => println!("EXEC or VFORK DONE"), //TODO: handle exec case
            PTRACE_S_SIGSTOP => println!("sigstop! {}", self.pid), //TODO: handle sigstop case
            _ => ()
        }
    }

    /// Standard handling of either:
    /// 1. the initial SIGTRAP signal
    /// 2. a syscall that is then translated
    fn handle_sigtrap_event(&mut self, info_bag: &mut InfoBag, signal: PtraceSignalEvent) {
        if signal == PTRACE_S_RAW_SIGTRAP {
            // it's the initial SIGTRAP signal
            self.set_ptrace_options(info_bag)
        }

        // This tracee got signaled then freed during the
        //  sysenter stage but the kernel reports the sysexit
        //  stage; just discard this spurious tracee/event.
        //if (tracee->exe == NULL) {
        //    tracee->restart_how = PTRACE_CONT;
        //    return 0;
        //}

        if self.seccomp {
            match self.status {
                TraceeStatus::SysEnter => {
                    // sysenter: ensure the sysexit stage will be hit under seccomp.
                    self.restart_how = Some(TraceeRestartMethod::WithExitStage);
                    self.sysexit_pending = true;
                }
                TraceeStatus::SysExit => {
                    // sysexit: the next sysenter will be notified by seccomp.
                    self.restart_how = Some(TraceeRestartMethod::WithoutExitStage);
                    self.sysexit_pending = false;
                }
            }
        }
        self.translate_syscall();
    }

    fn translate_syscall(&mut self) {
        // fetch_regs
        let regs: user_regs_struct = unsafe {fetch_regs(self.pid)};

        match self.status {
            TraceeStatus::SysEnter => {

                // save_current_regs(tracee, ORIGINAL);
                self.translate_syscall_enter(&regs);
                // save_current_regs(tracee, MODIFIED);

                //TODO: error handling/propagation (which requires removing expect() everywhere)
                /*
                /* Remember the tracee status for the "exit" stage and
                 * avoid the actual syscall if an error was reported
                 * by the translation/extension. */
                if (status < 0) {
                    set_sysnum(tracee, PR_void);
                    poke_reg(tracee, SYSARG_RESULT, (word_t) status);
                    tracee->status = status;
                }
                */

                self.status = TraceeStatus::SysExit;
            }
            TraceeStatus::SysExit => {
                self.status = TraceeStatus::SysEnter;
            }
        }

        // push_regs
    }

    fn translate_syscall_enter(&mut self, regs: &user_regs_struct) {
        //status = notify_extensions(tracee, SYSCALL_ENTER_START, 0, 0);

        //let sysnum = translate_sysnum(get_abi(tracee), peek_reg(tracee, version, SYSARG_NUM));
        let sysnum = get_reg!(regs, SysArgNum);
        println!("Sysnum : {:?}", sysnum);
    }

    fn handle_seccomp_event(&mut self, info_bag: &mut InfoBag, signal: PtraceSignalEvent) {
        println!("seccomp event! {:?}, {:?}", info_bag, signal);
    }

    fn new_child(&mut self, event: PtraceSignalEvent) {
        println!("new child: {:?}", event);
    }

    pub fn restart(&mut self) {
        match self.restart_how {
            Some(TraceeRestartMethod::WithoutExitStage) => ptrace(PTRACE_CONT, self.pid, null_mut(), null_mut())
                .expect("exit tracee without exit stage"),
            Some(TraceeRestartMethod::WithExitStage) => ptrace(PTRACE_SYSCALL, self.pid, null_mut(), null_mut())
                .expect("exit tracee with exit stage"),
            None => panic!("forgot to set restart method!")
        };

        // the restart method is reinitialised here
        self.restart_how = None;
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
    /// It requires a traced child process to be applied on,
    /// as using `ptrace(PTRACE_SETOPTIONS)` without preparation results in a Sys(ESRCH) error.
    fn create_set_ptrace_options() {
        match fork().expect("fork in set ptrace options tracee's test") {
            ForkResult::Parent { child } => {
                let info_bag = &mut InfoBag::new();
                let tracee = Tracee::new(child);
                assert_eq!(info_bag.deliver_sigtrap, false);

                // The parent will wait for the child's stop signal before calling set_ptrace_options
                assert_eq!(waitpid(-1, Some(__WALL)).expect("event loop waitpid"), Stopped(child, SIGSTOP));

                // This call must pass without panic
                tracee.set_ptrace_options(info_bag);

                // if everything went right, this boolean should have become true
                assert_eq!(info_bag.deliver_sigtrap, true);

                restart_and_end(child);
            }
            ForkResult::Child => {
                ptrace(PTRACE_TRACEME, 0, null_mut(), null_mut()).expect("test ptrace traceme");
                // we use a SIGSTOP to synchronise both processes
                kill(getpid(), SIGSTOP).expect("test child sigstop");
            }
        }
    }

    /// Restarts a child process, and waits/restarts it until it stops.
    fn restart_and_end(child: pid_t) {
        ptrace(PTRACE_SYSCALL, child, null_mut(), null_mut()).expect("exit tracee with exit stage");
        loop {
            match waitpid(-1, Some(__WALL)).expect("waitpid") {
                Exited(pid, exit_status) => {
                    assert_eq!(pid, child);

                    // the tracee should have exited with an OK status (exit code 0)
                    assert_eq!(exit_status, 0);
                    break;
                }
                _ => {
                    // restarting the tracee
                    ptrace(PTRACE_SYSCALL, child, null_mut(), null_mut()).expect("exit tracee with exit stage");
                }
            }
        }
    }
}