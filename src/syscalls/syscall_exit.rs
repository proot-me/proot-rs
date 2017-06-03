use syscalls::syscall_type::{SyscallType, syscall_type_from_sysnum};
use regs::regs_structs::user_regs_struct;
use syscalls::*;

pub fn translate_syscall_exit(regs: &user_regs_struct) {
    let sysnum = get_reg!(regs, SysArgNum) as usize;
    let systype = syscall_type_from_sysnum(sysnum);

    println!("exit  \t({:?}, \t{:?})", sysnum, systype);

    match systype {
        SyscallType::Brk                => brk::exit(),
        SyscallType::GetCwd             => getcwd::exit(),
        SyscallType::Accept             => accept::exit(),
        SyscallType::GetSockOrPeerName  => get_sockorpeer_name::exit(),
        SyscallType::SocketCall         => socketcall::exit(),
        SyscallType::Chdir              => chdir::exit(),
        SyscallType::Rename             => link_rename::exit(),
        SyscallType::RenameAt           => rename_at::exit(),
        SyscallType::ReadLink           => readlink_at::exit(),
        SyscallType::ReadLinkAt         => readlink_at::exit(),
        #[cfg(all(target_os="linux", target_arch="x86_64"))]
        SyscallType::Uname              => uname::exit(),
        SyscallType::Execve             => execve::exit(),
        SyscallType::Ptrace             => ptrace::exit(),
        SyscallType::Wait               => wait::exit(),
        _                               => {},
    }
}