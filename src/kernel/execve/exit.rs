use kernel::exit::SyscallExitResult;
use register::Registers;
use process::tracee::Tracee;

pub fn translate(tracee: &Tracee, regs: &Registers) -> SyscallExitResult {
    //TODO: implement ptrace execve exit translation

    let syscall_result = regs.sys_arg_result;

    println!(
        "execve exit: syscall result = {},  {:?}",
        syscall_result,
        tracee.new_exe
    );


    SyscallExitResult::Value(0)
}
