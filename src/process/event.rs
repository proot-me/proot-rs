use crate::process::proot::InfoBag;
use crate::process::tracee::{Tracee, TraceeRestartMethod, TraceeStatus};
use crate::process::translation::SyscallTranslator;
use nix::sys::ptrace::Event as PtraceEvent;

pub trait EventHandler {
    fn handle_syscall_stop_event(
        &mut self,
        info_bag: &mut InfoBag,
        #[cfg(test)] func_syscall_hook: &Option<Box<dyn Fn(&Tracee, bool, bool)>>,
    );
    fn handle_sigstop_event(&mut self);
    fn handle_seccomp_event(&mut self, info_bag: &mut InfoBag, event: PtraceEvent);
    fn handle_exec_vfork_event(&mut self);
    fn handle_new_child_event(&mut self, event: PtraceEvent);
}

impl EventHandler for Tracee {
    /// Standard handling of syscall-stop: translate the system call's
    /// parameters and restart it
    fn handle_syscall_stop_event(
        &mut self,
        info_bag: &mut InfoBag,
        #[cfg(test)] func_syscall_hook: &Option<Box<dyn Fn(&Tracee, bool, bool)>>,
    ) {
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
        self.translate_syscall(
            info_bag,
            #[cfg(test)]
            func_syscall_hook,
        );
    }

    fn handle_sigstop_event(&mut self) {
        debug!("sigstop! {}", self.pid);

        // Stop this tracee until PRoot has received
        // the EVENT_*FORK|CLONE notification.
        // if self.exe.is_none() {
        //     tracee->sigstop = SIGSTOP_PENDING;
        //     self.restart_how = TraceeRestartMethod::None;
        //     return TraceeRestartSignal::Stopped;
        // }
    }

    fn handle_seccomp_event(&mut self, info_bag: &mut InfoBag, signal: PtraceEvent) {
        debug!("seccomp event! {:?}, {:?}", info_bag, signal);
    }

    fn handle_exec_vfork_event(&mut self) {
        debug!("EXEC or VFORK event");
    }

    fn handle_new_child_event(&mut self, event: PtraceEvent) {
        debug!("new child: {:?}", event);
    }
}
