use kernel::groups::{SyscallGroup, syscall_group_from_sysnum};
use kernel::execve;
use kernel::heap::*;
use kernel::ptrace::*;
use kernel::socket::*;
use kernel::standard::*;
use libc::c_int;
use register::Registers;
use process::tracee::Tracee;

pub enum SyscallExitResult {
    /// The SYSARG_RESULT register will be poked and changed to the c_int value.
    Value(c_int),
    /// The SYSARG_RESULT register won't be changed.
    None,
}

impl SyscallExitResult {
    pub fn is_err(&self) -> bool {
        match *self {
            SyscallExitResult::None => false,
            SyscallExitResult::Value(_) => true,
        }
    }

    pub fn get_errno(&self) -> i32 {
        match *self {
            SyscallExitResult::None => 0,
            SyscallExitResult::Value(errno) => errno
        }
    }
}

pub fn translate(tracee: &Tracee, regs: &Registers) -> SyscallExitResult {
    let systype = syscall_group_from_sysnum(regs.sys_num);

    println!("exit  \t({:?}, \t{:?})", regs.sys_num, systype);

    match systype {
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
        SyscallGroup::Execve => execve::exit(tracee, regs),
        SyscallGroup::Ptrace => ptrace::exit(),
        SyscallGroup::Wait => wait::exit(),
        _ => SyscallExitResult::None,
    }
}
