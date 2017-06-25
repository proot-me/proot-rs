use syscalls::syscall_exit::SyscallExitResult;
use nix::Result;

pub fn enter() -> Result<()> {
    Ok(())

}

pub fn exit() -> SyscallExitResult {
    SyscallExitResult::Value(0)
}