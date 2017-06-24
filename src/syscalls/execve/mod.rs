pub mod enter;
pub mod exit;

use syscalls::syscall_exit::SyscallExitResult;

pub fn enter() {
    enter::translate()
}

pub fn exit() -> SyscallExitResult {
    exit::translate()
}