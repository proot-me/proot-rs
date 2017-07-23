use kernel::exit::SyscallExitResult;

pub fn translate() -> SyscallExitResult {
    SyscallExitResult::Value(0)
}
