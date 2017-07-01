use nix::Result;
use syscalls::syscall_exit::SyscallExitResult;

pub fn enter() -> Result<()> {
    Ok(())
}

pub fn exit() -> SyscallExitResult {
    SyscallExitResult::Value(0)
}