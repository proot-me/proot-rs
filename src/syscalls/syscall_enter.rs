use syscalls::syscall_type::{SyscallType, syscall_type_from_sysnum};
use regs::regs_structs::user_regs_struct;
use syscalls::*;
use nix::Result;

pub fn translate(regs: &user_regs_struct) -> Result<()> {
    let sysnum = get_reg!(regs, SysArgNum) as usize;
    let systype = syscall_type_from_sysnum(sysnum);

    println!("enter  \t({:?}, \t{:?}) ", sysnum, systype);

    match systype {
        SyscallType::Accept             => accept::enter(),
        SyscallType::BindConnect        => bind_connect::enter(),
        SyscallType::Brk                => brk::enter(),
        SyscallType::Chdir              => chdir::enter(),
        SyscallType::ChmodAccessMkNodAt => chmod_access_mknod_at::enter(),
        SyscallType::DirLinkAttr        => dir_link_attr::enter(),
        SyscallType::Execve             => execve::enter(),
        SyscallType::GetCwd             => getcwd::enter(),
        SyscallType::GetSockOrPeerName  => get_sockorpeer_name::enter(),
        SyscallType::InotifyAddWatch    => inotify_add_watch::enter(),
        SyscallType::Link               => link_rename::enter(),
        SyscallType::LinkAt             => link_at::enter(),
        SyscallType::Mount              => mount::enter(),
        SyscallType::Open               => open::enter(),
        SyscallType::OpenAt             => open_at::enter(),
        SyscallType::PivotRoot          => pivot_root::enter(),
        SyscallType::Ptrace             => ptrace::enter(),
        SyscallType::ReadLink           => standard_syscall::enter(),
        SyscallType::ReadLinkAt         => readlink_at::enter(),
        SyscallType::Rename             => link_rename::enter(),
        SyscallType::RenameAt           => rename_at::enter(),
        SyscallType::SocketCall         => socketcall::enter(),
        SyscallType::StandardSyscall    => standard_syscall::enter(),
        SyscallType::StatAt             => stat_at::enter(),
        SyscallType::SymLink            => sym_link::enter(),
        SyscallType::SymLinkAt          => sym_link_at::enter(),
        SyscallType::Wait               => wait::enter(),
        SyscallType::UnlinkMkdirAt      => unlink_mkdir_at::enter(),
        _                               => {}
    }

    Ok(())
}