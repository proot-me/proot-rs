use crate::errors::Result;
use crate::kernel::exit::SyscallExitResult;
use crate::process::tracee::Tracee;
use crate::register::{Current, SysResult};

pub fn translate(tracee: &mut Tracee) -> SyscallExitResult {
    let syscall_result = tracee.regs.get(Current, SysResult) as isize;

    //TODO: implement ptrace execve exit translation

    if syscall_result < 0 {
        return SyscallExitResult::None;
    }

    if tracee.new_exe.is_some() {
        // Execve happened; commit the new "/proc/self/exe".
        tracee.exe = tracee.new_exe.take();
    }

    //TODO: implement heap
    // New processes have no heap.
    //bzero(tracee->heap, sizeof(Heap));

    match transfert_load_script(tracee) {
        Err(error) => SyscallExitResult::Error(error),
        Ok(()) => SyscallExitResult::None,
    }
}

pub fn transfert_load_script(_tracee: &mut Tracee) -> Result<()> {
    Ok(())
}
