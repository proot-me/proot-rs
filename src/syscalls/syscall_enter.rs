use syscalls::syscalltype::{SyscallType, syscall_type_from_sysnum};
use regs::regs_structs::user_regs_struct;
use syscalls::*;

pub fn translate_syscall_enter(regs: &user_regs_struct) {
    let sysnum = get_reg!(regs, SysArgNum) as usize;
    let systype = syscall_type_from_sysnum(sysnum);

    match systype {
        SyscallType::Execve             => execve::enter(),
        SyscallType::Ptrace             => ptrace::enter(),
        SyscallType::Wait               => wait::enter(),
        SyscallType::Brk                => brk::enter(),
        SyscallType::GetCwd             => getcwd::enter(),
        SyscallType::Chdir              => chdir::enter(),
        SyscallType::BindConnect        => bind_connect::enter(),
        SyscallType::Accept             => accept::enter(),
        SyscallType::GetSockOrPeerName  => get_sockorpeer_name::enter(),
        SyscallType::SocketCall         => socketcall::enter(),
        SyscallType::StandardSyscall    => standard_syscall::enter(sysnum),
        SyscallType::Open               => open::enter(),
        SyscallType::StatAt             => stat_at::enter(),
        SyscallType::ChmodAccessMkNodAt => chmod_access_mknod_at::enter(),
        SyscallType::InotifyAddWatch    => inotify_add_watch::enter(),
        SyscallType::DirLinkAttr        => dir_link_attr::enter(),
        SyscallType::PivotRoot          => pivot_root::enter(),
        SyscallType::LinkAt             => link_at::enter(),
        SyscallType::Mount              => mount::enter(),
        SyscallType::OpenAt             => open_at::enter(),
        SyscallType::DirLinkAt          => dir_link_at::enter(),
        SyscallType::LinkRename         => link_rename::enter(),
        SyscallType::RenameAt           => rename_at::enter(),
        SyscallType::SymLink            => sym_link::enter(),
        SyscallType::SymLinkAt          => sym_link_at::enter(),
        SyscallType::Ignored            => println!("ignored {:?}", sysnum)
    }
}