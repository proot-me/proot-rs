use errors::Result;
use register::{Word, Registers, SysResult, Original, Current, Modified, StackPointer};
use kernel::{enter, exit};
use kernel::exit::SyscallExitResult;
use process::proot::InfoBag;
use process::tracee::{TraceeStatus, TraceeRestartMethod, Tracee};

pub trait SyscallTranslator {
    fn translate_syscall(&mut self, info_bag: &InfoBag);
    fn translate_syscall_enter(&mut self, info_bag: &InfoBag) -> Result<()>;
    fn translate_syscall_exit(&mut self);
}

impl SyscallTranslator for Tracee {
    /// Retrieves the registers,
    /// handles either the enter or exit stage of the system call,
    /// and pushes the registers.
    fn translate_syscall(&mut self, info_bag: &InfoBag) {
        // We retrieve the registers of the current tracee.
        // They contain the system call's number, arguments and other register's info.
        if let Err(error) = self.regs.fetch_regs() {
            eprintln!("proot error: Error while fetching regs: {}", error);
            return;
        }

        match self.status {
            TraceeStatus::SysEnter => {
                // Never restore original register values at the end of this stage.
                self.regs.set_restore_original_regs(false);

                // Saving the original registers here.
                // It is paramount in order to restore the regs after the exit stage,
                // and also as memory in order to remember the original values (like
                // the syscall number).
                self.regs.save_current_regs(Original);

                let status = self.translate_syscall_enter(info_bag);

                // Saving the registers potentially modified by the translation.
                // It's useful in order to know what the translation did to the registers.
                self.regs.save_current_regs(Modified);

                // In case of error reported by the translation/extension,
                // remember the tracee status for the "exit" stage and avoid
                // the actual syscall.
                if status.is_err() {
                    self.regs.cancel_syscall("following error during enter stage, avoid syscall");
                    self.regs.set(SysResult,
                        status.unwrap_err().get_errno() as Word,
                        "following error during enter stage, remember errno for exit stage",
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
                    self.regs.restore_original(
                        StackPointer,
                        "following enter stage, restoring stack pointer early because no exit stage"
                    );
                }
            }
            TraceeStatus::SysExit |
            TraceeStatus::Error(_) => {
                // By default, restore original register values at the end of this stage.
                self.regs.set_restore_original_regs(true);

                self.translate_syscall_exit();

                // reset the tracee's status
                self.status = TraceeStatus::SysEnter;
            }
        }

        if let Err(error) = self.regs.push_regs() {
            eprintln!("proot error: Error while pushing regs: {}", error);
        }
    }

    fn translate_syscall_enter(&mut self, info_bag: &InfoBag) -> Result<()> {
        //TODO: notify extensions for SYSCALL_ENTER_START
        // status = notify_extensions(tracee, SYSCALL_ENTER_START, 0, 0);
        // if (status < 0)
        //     goto end;
        // if (status > 0)
        //     return 0;

        let status = enter::translate(info_bag, self);

        //TODO: notify extensions for SYSCALL_ENTER_END event
        // status2 = notify_extensions(tracee, SYSCALL_ENTER_END, status, 0);
        // if (status2 < 0)
        //     status = status2;

        status
    }

    fn translate_syscall_exit(&mut self) {
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
            match exit::translate(self) {
                // The syscall result won't be altered.
                SyscallExitResult::None => (),
                // The syscall result will be modified. This is not an error.
                SyscallExitResult::Value(value) => {
                    self.regs.set(
                        SysResult,
                        value as Word,
                        "following exit translation, setting new syscall result",
                    )
                }
                // The syscall result will be modified. This is an error.
                SyscallExitResult::Error(error) => {
                    self.regs.set(
                        SysResult,
                        // errno is negative
                        error.get_errno() as Word,
                        "following error during exit translation, setting errno",
                    )
                }
            };
        } else {
            self.regs.set(
                SysResult,
                self.status.get_errno() as Word,
                "following previous error in enter stage, setting errno",
            );
        }

        //TODO: notify extensions for SYSCALL_EXIT_END event
        // status = notify_extensions(tracee, SYSCALL_EXIT_END, 0, 0);
        // if (status < 0)
        //     poke_reg(tracee, SYSARG_RESULT, (word_t) status);
    }
}
