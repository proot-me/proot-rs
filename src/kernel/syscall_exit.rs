use kernel::syscall_type::{SyscallType, syscall_type_from_sysnum};
use kernel::execve;
use kernel::heap::*;
use kernel::ptrace::*;
use kernel::socket::*;
use kernel::standard::*;
use libc::{c_int, user_regs_struct};

pub enum SyscallExitResult {
    /// The SYSARG_RESULT register will be poked and changed to the c_int value.
    Value(c_int),
    /// The SYSARG_RESULT register won't be changed.
    None,
}

impl SyscallExitResult {
    pub fn is_none(&self) -> bool {
        match *self {
            SyscallExitResult::None => true,
            _ => false,
        }
    }

    /*
    pub fn get_value(&self) -> c_int {
        match *self {
            SyscallExitResult::Value(value) => value,
            SyscallExitResult::None => panic!("asked for value, but syscall exit result is none")
        }
    }
    */
}

pub fn translate(regs: &user_regs_struct) -> SyscallExitResult {
    let sysnum = get_reg!(regs, SysArgNum) as usize;
    let systype = syscall_type_from_sysnum(sysnum);

    println!("exit  \t({:?}, \t{:?})", sysnum, systype);

    match systype {
        SyscallType::Brk => brk::exit(),
        SyscallType::GetCwd => getcwd::exit(),
        SyscallType::Accept => accept::exit(),
        SyscallType::GetSockOrPeerName => get_sockorpeer_name::exit(),
        SyscallType::SocketCall => socketcall::exit(),
        SyscallType::Chdir => chdir::exit(),
        SyscallType::Rename => link_rename::exit(),
        SyscallType::RenameAt => rename_at::exit(),
        SyscallType::ReadLink => readlink_at::exit(),
        SyscallType::ReadLinkAt => readlink_at::exit(),
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        SyscallType::Uname => uname::exit(),
        SyscallType::Execve => execve::exit(),
        SyscallType::Ptrace => ptrace::exit(),
        SyscallType::Wait => wait::exit(),
        _ => SyscallExitResult::None,
    }
}
