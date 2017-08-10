use errors::Result;
use register::{Word, Registers, SysResult};
use kernel::{enter, exit};
use kernel::exit::SyscallExitResult;
use process::proot::InfoBag;
use process::tracee::{TraceeStatus, TraceeRestartMethod, Tracee};

pub trait SyscallTranslator {
    fn translate_syscall(&mut self, info_bag: &InfoBag);
    fn translate_syscall_enter(&mut self, info_bag: &InfoBag, regs: &mut Registers) -> Result<()>;
    fn translate_syscall_exit(&mut self, regs: &mut Registers);
}

impl SyscallTranslator for Tracee {
    /// Retrieves the registers,
    /// handles either the enter or exit stage of the system call,
    /// and pushes the registers.
    fn translate_syscall(&mut self, info_bag: &InfoBag) {
        // We retrieve the registers of the current tracee.
        // They contain the system call's number, arguments and other register's info.
        let mut regs = match Registers::fetch_regs(self.pid) {
            Ok(regs) => regs,
            Err(_) => return,
        };

        match self.status {
            TraceeStatus::SysEnter => {
                // Never restore original register values at the end of this stage.
                regs.push_only_result(false);

                let status = self.translate_syscall_enter(info_bag, &mut regs);

                if status.is_err() {
                    // Remember the tracee status for the "exit" stage and
                    // avoid the actual syscall if an error was reported
                    // by the translation/extension.
                    regs.void_syscall();
                    regs.set(
                        SysResult,
                        status.unwrap_err().get_errno() as Word,
                        "setting errno because of an error that occurred during enter translation",
                    );
                    self.status = TraceeStatus::Error(status.unwrap_err());
                } else {
                    self.status = TraceeStatus::SysExit;
                }

                // Restore tracee's stack pointer now if it won't hit
                // the sysexit stage (i.e. when seccomp is enabled and
                // there's nothing else to do).
                if self.restart_how == TraceeRestartMethod::WithoutExitStage {
                    self.status = TraceeStatus::SysEnter;
                    regs.restore_stack_pointer(None);
                }
            }
            TraceeStatus::SysExit |
            TraceeStatus::Error(_) => {
                // By default, restore original register values at the end of this stage.
                regs.push_only_result(true);

                self.translate_syscall_exit(&mut regs);

                // reset the tracee's status
                self.status = TraceeStatus::SysEnter;
            }
        }

        if let Err(error) = regs.push_regs() {
            eprintln!("proot error: Error while pushing regs: {}", error);
        }

        // Saving the registers of the sys enter stage,
        // as these are useful for the sys exit stage translation.
        if self.status == TraceeStatus::SysExit {
            self.saved_regs = Some(regs);
        }
    }

    fn translate_syscall_enter(&mut self, info_bag: &InfoBag, regs: &mut Registers) -> Result<()> {
        //TODO: notify extensions for SYSCALL_ENTER_START
        // status = notify_extensions(tracee, SYSCALL_ENTER_START, 0, 0);
        // if (status < 0)
        //     goto end;
        // if (status > 0)
        //     return 0;

        let status = enter::translate(info_bag, self, regs);

        //TODO: notify extensions for SYSCALL_ENTER_END event
        // status2 = notify_extensions(tracee, SYSCALL_ENTER_END, status, 0);
        // if (status2 < 0)
        //     status = status2;

        status
    }

    fn translate_syscall_exit(&mut self, regs: &mut Registers) {
        //TODO: notify extensions for SYSCALL_EXIT_START event
        // status = notify_extensions(tracee, SYSCALL_EXIT_START, 0, 0);
        // if (status < 0) {
        //     poke_reg(tracee, SYSARG_RESULT, (word_t) status);
        //     goto end;
        // }
        // if (status > 0)
        //     return;

        if self.status.is_ok() {
            // the exit stage translation happens now
            match exit::translate(self, regs) {
                SyscallExitResult::None => (), // do not alter the result,
                SyscallExitResult::Value(value) => {
                    regs.set(
                        SysResult,
                        value as Word,
                        "setting new syscall result after exit translation",
                    )
                }
                SyscallExitResult::Error(error) => {
                    regs.set(
                        SysResult,
                        // errno is negative
                        error.get_errno() as Word,
                        "setting errno because of an error that occurred during exit translation",
                    )
                }
            };
        } else {
            regs.set(
                SysResult,
                self.status.get_errno() as Word,
                "setting errno because of an error that occurred previously",
            );
        }

        //TODO: notify extensions for SYSCALL_EXIT_END event
        // status = notify_extensions(tracee, SYSCALL_EXIT_END, 0, 0);
        // if (status < 0)
        //     poke_reg(tracee, SYSARG_RESULT, (word_t) status);
    }
}
