pub mod enter;
pub mod exit;
pub mod shebang;

use libc::pid_t;
use errors::Result;
use register::Registers;
use kernel::syscall_exit::SyscallExitResult;
use filesystem::fs::FileSystem;

pub fn enter(pid: pid_t, fs: &FileSystem, regs: &Registers) -> Result<()> {
    enter::translate(pid, fs, regs)
}

pub fn exit() -> SyscallExitResult {
    exit::translate()
}
