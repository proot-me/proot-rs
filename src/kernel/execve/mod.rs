pub mod enter;
pub mod exit;
pub mod path;
pub mod shebang;

use libc::{pid_t, user_regs_struct};
use nix::Result;
use kernel::syscall_exit::SyscallExitResult;

pub fn enter(pid: pid_t, regs: &user_regs_struct) -> Result<()> {
    enter::translate(pid, regs)
}

pub fn exit() -> SyscallExitResult {
    exit::translate()
}
