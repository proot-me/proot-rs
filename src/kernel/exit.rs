use crate::errors::Error;
use crate::kernel::execve;
use crate::kernel::groups::{syscall_group_from_sysnum, SyscallGroup};
use crate::kernel::heap::*;
use crate::kernel::ptrace::*;
use crate::kernel::socket::*;
use crate::kernel::standard::*;
use crate::process::tracee::Tracee;
use crate::register::{Current, SysResult, Word};

#[allow(dead_code)]
pub enum SyscallExitResult {
    /// The SYS_RESULT register won't be overwritten.
    None,
    /// Indicates a new value for the syscall result, that is not an error.
    /// The SYS_RESULT register will be poked and changed to the new value.
    Value(Word),
    /// Indicates an error that happened during the translation.
    /// The SYS_RESULT register will be poked and changed to the new value.
    /// More precisely, the new value will be `-errno`.
    Error(Error),
}

pub fn translate(tracee: &mut Tracee) {
    let syscall_number = tracee.regs.get_sys_num(Current);
    let syscall_group = syscall_group_from_sysnum(syscall_number);

    debug!("Syscall exit ({:?}, {:?})", syscall_number, syscall_group);

    let result = match syscall_group {
        SyscallGroup::Brk => brk::exit(),
        SyscallGroup::GetCwd => getcwd::exit(),
        SyscallGroup::Accept => accept::exit(),
        SyscallGroup::GetSockOrPeerName => get_sockorpeer_name::exit(),
        SyscallGroup::SocketCall => socketcall::exit(),
        SyscallGroup::Chdir => chdir::exit(),
        SyscallGroup::Rename => link_rename::exit(),
        SyscallGroup::RenameAt => rename_at::exit(),
        SyscallGroup::ReadLink => readlink_at::exit(),
        SyscallGroup::ReadLinkAt => readlink_at::exit(),
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        SyscallGroup::Uname => uname::exit(),
        SyscallGroup::Execve => execve::exit(tracee),
        SyscallGroup::Ptrace => ptrace::exit(),
        SyscallGroup::Wait => wait::exit(),
        _ => SyscallExitResult::None,
    };

    match result {
        SyscallExitResult::None => (),
        SyscallExitResult::Value(value) => tracee.regs.set(
            SysResult,
            value as Word,
            "following exit translation, setting new syscall result",
        ),
        SyscallExitResult::Error(error) => {
            tracee.regs.set(
                SysResult,
                // errno is negative
                error.get_errno() as Word,
                "following error during exit translation, setting errno",
            )
        }
    };
}
