use crate::kernel::execve;
use crate::kernel::groups::{syscall_group_from_sysnum, SyscallGroup};
use crate::kernel::heap::*;
use crate::kernel::ptrace::*;
use crate::kernel::socket::*;
use crate::kernel::standard::*;
use crate::process::tracee::Tracee;
use crate::register::{Original, SysResult, Word};

pub fn translate(tracee: &mut Tracee) {
    let syscall_number = tracee.regs.get_sys_num(Original);
    let syscall_group = syscall_group_from_sysnum(syscall_number);

    trace!("Syscall exit ({:?}, {:?})", syscall_number, syscall_group);

    let result = match syscall_group {
        SyscallGroup::Brk => brk::exit(),
        SyscallGroup::GetCwd => getcwd::exit(tracee),
        SyscallGroup::Accept => accept::exit(),
        SyscallGroup::GetSockOrPeerName => get_sockorpeer_name::exit(),
        SyscallGroup::SocketCall => socketcall::exit(),
        SyscallGroup::Chdir => chdir::exit(tracee),
        SyscallGroup::Rename => link_rename::exit(),
        SyscallGroup::RenameAt => rename_at::exit(),
        SyscallGroup::ReadLinkAt => readlink_at::exit(),
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        SyscallGroup::Uname => uname::exit(),
        SyscallGroup::Execve => execve::exit(tracee),
        SyscallGroup::Ptrace => ptrace::exit(),
        SyscallGroup::Wait => wait::exit(),
        _ => Ok(()),
    };

    if let Err(error) = result {
        debug!("syscall translate raised an error: {:?}", error);
        tracee.regs.set(
            SysResult,
            // errno is negative
            error.get_errno() as Word,
            "following error during exit translation, setting errno",
        );
    };
}
