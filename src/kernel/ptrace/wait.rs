use kernel::exit::SyscallExitResult;
use errors::Result;

pub fn enter() -> Result<()> {
    Ok(())

}

pub fn exit() -> SyscallExitResult {
    SyscallExitResult::Value(0)
}
