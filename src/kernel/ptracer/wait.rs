use errors::Result;
use kernel::exit::SyscallExitResult;

pub fn enter() -> Result<()> {
    Ok(())
}

pub fn exit() -> SyscallExitResult {
    SyscallExitResult::None
}
