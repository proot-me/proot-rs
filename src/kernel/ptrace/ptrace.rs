use errors::Result;
use kernel::syscall_exit::SyscallExitResult;

pub fn enter() -> Result<()> {
    Ok(())
}

pub fn exit() -> SyscallExitResult {
    SyscallExitResult::Value(0)
}
