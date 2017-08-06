use errors::Result;
use register::Registers;
use kernel::{enter, exit};
use process::proot::InfoBag;
use process::tracee::{TraceeStatus, TraceeRestartMethod, Tracee};

pub trait SyscallTranslator {
    fn translate_syscall(&mut self, info_bag: &InfoBag);
    fn translate_syscall_enter(&mut self, info_bag: &InfoBag, regs: &mut Registers) -> Result<()>;
    fn translate_syscall_exit(&mut self, regs: &Registers);
}

impl SyscallTranslator for Tracee {
    /// Retrieves the registers,
    /// handles either the enter or exit stage of the system call,
    /// and pushes the registers.
    fn translate_syscall(&mut self, info_bag: &InfoBag) {
        // We retrieve the registers of the current tracee.
        // They contain the system call's number, arguments and other register's info.
        let mut regs = match Registers::retrieve(self.pid) {
            Ok(regs) => regs,
            Err(_) => return,
        };

        match self.status {
            TraceeStatus::SysEnter => {
                /* Never restore original register values at the end
                 * of this stage.  */
                // tracee->restore_original_regs = false;

                // save_current_regs(tracee, ORIGINAL);
                let status = self.translate_syscall_enter(info_bag, &mut regs);
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
                if self.restart_how == TraceeRestartMethod::WithoutExitStage {
                    self.status = TraceeStatus::SysEnter;
                    // poke_reg(tracee, STACK_POINTER, peek_reg(tracee, ORIGINAL, STACK_POINTER));
                }
            }
            TraceeStatus::SysExit |
            TraceeStatus::Error(_) => {
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

    fn translate_syscall_enter(&mut self, info_bag: &InfoBag, regs: &mut Registers) -> Result<()> {
        // status = notify_extensions(tracee, SYSCALL_ENTER_START, 0, 0);
        // if (status < 0)
        //     goto end;
        // if (status > 0)
        //     return 0;

        let status = enter::translate(info_bag, self, regs);

        // status2 = notify_extensions(tracee, SYSCALL_ENTER_END, status, 0);
        // if (status2 < 0)
        //     status = status2;

        status
    }

    fn translate_syscall_exit(&mut self, regs: &Registers) {
        // status = notify_extensions(tracee, SYSCALL_EXIT_START, 0, 0);
        // if (status < 0) {
        //     poke_reg(tracee, SYSARG_RESULT, (word_t) status);
        //     goto end;
        // }
        // if (status > 0)
        //     return;

        // Set the tracee's errno if an error occurred previously during the translation.
        if self.status.is_err() {
            // poke_reg(tracee, SYSARG_RESULT, (word_t) tracee->status);
        } else {
            let syscall_exit_result = exit::translate(self, regs);

            if !syscall_exit_result.is_none() {
                // poke_reg(tracee, SYSARG_RESULT, (word_t) status.get_value());
            }
        }

        // status = notify_extensions(tracee, SYSCALL_EXIT_END, 0, 0);
        // if (status < 0)
        //     poke_reg(tracee, SYSARG_RESULT, (word_t) status);
    }
}
