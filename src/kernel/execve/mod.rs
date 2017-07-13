pub mod enter;
pub mod exit;
pub mod shebang;

use libc::{pid_t, user_regs_struct};
use nix::Result;
use kernel::syscall_exit::SyscallExitResult;
use filesystem::fs::FileSystem;

pub fn enter(pid: pid_t, fs: &FileSystem, regs: &user_regs_struct) -> Result<()> {
    enter::translate(pid, fs, regs)
}

pub fn exit() -> SyscallExitResult {
    exit::translate()
}
