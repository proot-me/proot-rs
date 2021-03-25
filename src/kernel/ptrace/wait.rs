use crate::errors::Result;
use crate::kernel::exit::SyscallExitResult;

pub fn enter() -> Result<()> {
    Ok(())
}

pub fn exit() -> SyscallExitResult {
    SyscallExitResult::None
}
