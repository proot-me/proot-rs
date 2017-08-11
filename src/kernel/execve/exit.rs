use kernel::exit::SyscallExitResult;
use register::{Registers, SysResult, Current};
use process::tracee::Tracee;

pub fn translate(tracee: &Tracee) -> SyscallExitResult {
    //TODO: implement ptrace execve exit translation

    let syscall_result = tracee.regs.get(Current, SysResult);

    println!(
        "execve exit: syscall result = {},  {:?}",
        syscall_result,
        tracee.new_exe
    );

    SyscallExitResult::None
}
