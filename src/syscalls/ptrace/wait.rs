use syscalls::syscall_exit::SyscallExitResult;

pub fn enter() {
}

pub fn exit() -> SyscallExitResult {
    SyscallExitResult::Value(0)
}