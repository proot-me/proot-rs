use kernel::syscall_exit::SyscallExitResult;

pub fn translate() -> SyscallExitResult {
    SyscallExitResult::Value(0)
}
