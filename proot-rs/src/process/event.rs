use std::cell::RefCell;
use std::rc::Rc;

use libc::pid_t;
use nix::sched::CloneFlags;
use nix::sys::ptrace::Event as PtraceEvent;
use nix::unistd::Pid;

use crate::errors::*;
use crate::filesystem::FileSystem;
use crate::process::proot::InfoBag;
use crate::process::tracee::{Tracee, TraceeRestartMethod, TraceeStatus};
use crate::process::translation::SyscallTranslator;
use crate::register::{RegVersion, SysArg, SysArg1};

use super::tracee::SigStopStatus;

pub trait EventHandler {
    fn handle_syscall_stop_event(
        &mut self,
        info_bag: &mut InfoBag,
        #[cfg(test)] func_syscall_hook: &Option<Box<dyn Fn(&Tracee, bool, bool)>>,
    );
    fn handle_sigstop_event(&mut self);
    fn handle_seccomp_event(&mut self, info_bag: &mut InfoBag, event: PtraceEvent);
    fn handle_exec_vfork_event(&mut self);
    fn handle_new_child_event(&mut self) -> Result<Tracee>;
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

    fn handle_new_child_event(&mut self) -> Result<Tracee> {
        // We need to fetch the values of the registers to determine the flags when the
        // child processes is spawned.
        self.regs.fetch_regs()?;
        let sysnum = self.regs.get_sys_num(RegVersion::Current);

        let clone_flags = match sysnum {
            #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
            sc::nr::VFORK => CloneFlags::CLONE_VFORK,
            sc::nr::CLONE => CloneFlags::from_bits_truncate(
                self.regs.get(RegVersion::Current, SysArg(SysArg1)) as i32,
            ),
            _ => CloneFlags::empty(),
        };

        // Get the pid of the parent's new child.
        let child_pid = Pid::from_raw(nix::sys::ptrace::getevent(self.pid)? as pid_t);

        // TODO: CLONE_VM
        // child->verbose = parent->verbose;
        // child->seccomp = parent->seccomp;
        // child->sysexit_pending = parent->sysexit_pending;

        // TALLOC_FREE(child->heap);
        // child->heap = ((clone_flags & CLONE_VM) != 0)
        //     ? talloc_reference(child, parent->heap)
        //     : talloc_memdup(child, parent->heap, sizeof(Heap));
        // if (child->heap == NULL)
        //     return -ENOMEM;

        // TODO: CLONE_PARENT
        // if ((clone_flags & CLONE_PARENT) != 0)
        //     child->parent = parent->parent;
        // else
        //     child->parent = parent;

        // TODO: CLONE_THREAD
        // child->clone = ((clone_flags & CLONE_THREAD) != 0);

        // TODO: emulate ptrace()
        // /* Depending on how the new process is created, it may be
        // * automatically traced by the parent's tracer.  */
        // ptrace_options = ( clone_flags == 0			? PTRACE_O_TRACEFORK
        //         : (clone_flags & 0xFF) == SIGCHLD	? PTRACE_O_TRACEFORK
        //         : (clone_flags & CLONE_VFORK) != 0	? PTRACE_O_TRACEVFORK
        //         : 					  PTRACE_O_TRACECLONE);
        // if (parent->as_ptracee.ptracer != NULL
        //     && (   (ptrace_options & parent->as_ptracee.options) != 0
        //     || (clone_flags & CLONE_PTRACE) != 0)) {
        //     attach_to_ptracer(child, parent->as_ptracee.ptracer);

        //     /* All these flags are inheritable, no matter why this
        //     * child is being traced.  */
        //     child->as_ptracee.options |= (parent->as_ptracee.options
        //                     & ( PTRACE_O_TRACECLONE
        //                     | PTRACE_O_TRACEEXEC
        //                     | PTRACE_O_TRACEEXIT
        //                     | PTRACE_O_TRACEFORK
        //                     | PTRACE_O_TRACESYSGOOD
        //                     | PTRACE_O_TRACEVFORK
        //                     | PTRACE_O_TRACEVFORKDONE));
        // }

        let fs = if clone_flags.contains(CloneFlags::CLONE_FS) {
            // share the same FileSystem instance
            self.fs.clone()
        } else {
            let fs: FileSystem = self.fs.borrow().clone();
            Rc::new(RefCell::new(fs))
        };
        let mut child_tracee = Tracee::new(child_pid, fs);

        // The path to the executable is unshared only once the child process does a
        // call to execve(2).
        child_tracee.exe = self.exe.clone();

        // child->qemu = talloc_reference(child, parent->qemu);
        // child->glue = talloc_reference(child, parent->glue);

        // child->host_ldso_paths  = talloc_reference(child, parent->host_ldso_paths);
        // child->guest_ldso_paths = talloc_reference(child, parent->guest_ldso_paths);

        // child->tool_name = parent->tool_name;

        // inherit_extensions(child, parent, clone_flags);

        // /* Restart the child tracee if it was already alive but
        // * stopped until that moment.  */
        // if (child->sigstop == SIGSTOP_PENDING) {
        //     bool keep_stopped = false;

        //     child->sigstop = SIGSTOP_ALLOWED;

        //     /* Notify its ptracer if it is ready to be traced.  */
        //     if (child->as_ptracee.ptracer != NULL) {
        //         /* Sanity check.  */
        //         assert(!child->as_ptracee.tracing_started);

        //         keep_stopped = handle_ptracee_event(child, __W_STOPCODE(SIGSTOP));

        //         /* Note that this event was already handled by
        //         * PRoot since child->as_ptracee.ptracer was
        //         * NULL up to now.  */
        //         child->as_ptracee.event4.proot.pending = false;
        //         child->as_ptracee.event4.proot.value   = 0;
        //     }

        //     if (!keep_stopped)
        //         (void) restart_tracee(child, 0);
        // }

        child_tracee.sigstop_status = SigStopStatus::WaitForSigStopClone;

        Ok(child_tracee)
    }
}
