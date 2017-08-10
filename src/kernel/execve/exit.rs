use kernel::exit::SyscallExitResult;
use register::{Registers, SysResult};
use process::tracee::Tracee;

pub fn translate(tracee: &Tracee, regs: &Registers) -> SyscallExitResult {
    //TODO: implement ptrace execve exit translation

    let syscall_result = regs.get(SysResult);

    println!(
        "execve exit: syscall result = {},  {:?}",
        syscall_result,
        tracee.new_exe
    );

    SyscallExitResult::Value(0)
}
