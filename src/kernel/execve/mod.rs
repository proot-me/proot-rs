pub mod enter;
pub mod exit;
pub mod shebang;
pub mod elf;
pub mod load_info;

use nix::unistd::Pid;
use errors::Result;
use register::Registers;
use kernel::syscall_exit::SyscallExitResult;
use filesystem::fs::FileSystem;
use process::tracee::Tracee;

pub fn enter(pid: Pid, fs: &FileSystem, tracee: &mut Tracee, regs: &Registers) -> Result<()> {
    enter::translate(pid, fs, tracee, regs)
}

pub fn exit() -> SyscallExitResult {
    exit::translate()
}
