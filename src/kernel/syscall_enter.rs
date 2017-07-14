use kernel::syscall_type::{SyscallType, syscall_type_from_sysnum};
use libc::pid_t;
use kernel::execve;
use kernel::heap::*;
use kernel::ptrace::*;
use kernel::socket::*;
use kernel::standard::*;
use nix::Result;
use filesystem::fs::FileSystem;
use register::Registers;

pub fn translate(pid: pid_t, fs: &FileSystem, regs: &Registers) -> Result<()> {
    let sys_type = syscall_type_from_sysnum(regs.sys_num);

    println!("enter  \t({:?}, \t{:?}) ", regs.sys_num, sys_type);

    match sys_type {
        SyscallType::Accept => accept::enter(),
        SyscallType::BindConnect => bind_connect::enter(),
        SyscallType::Brk => brk::enter(),
        SyscallType::Chdir => chdir::enter(),
        SyscallType::ChmodAccessMkNodAt => chmod_access_mknod_at::enter(),
        SyscallType::DirLinkAttr => dir_link_attr::enter(),
        SyscallType::Execve => execve::enter(pid, fs, regs),
        SyscallType::GetCwd => getcwd::enter(),
        SyscallType::GetSockOrPeerName => get_sockorpeer_name::enter(),
        SyscallType::InotifyAddWatch => inotify_add_watch::enter(),
        SyscallType::Link => link_rename::enter(),
        SyscallType::LinkAt => link_at::enter(),
        SyscallType::Mount => mount::enter(),
        SyscallType::Open => open::enter(),
        SyscallType::OpenAt => open_at::enter(),
        SyscallType::PivotRoot => pivot_root::enter(),
        SyscallType::Ptrace => ptrace::enter(),
        SyscallType::ReadLink => standard_syscall::enter(),
        SyscallType::ReadLinkAt => readlink_at::enter(),
        SyscallType::Rename => link_rename::enter(),
        SyscallType::RenameAt => rename_at::enter(),
        SyscallType::SocketCall => socketcall::enter(),
        SyscallType::StandardSyscall => standard_syscall::enter(),
        SyscallType::StatAt => stat_at::enter(),
        SyscallType::SymLink => sym_link::enter(),
        SyscallType::SymLinkAt => sym_link_at::enter(),
        SyscallType::Wait => wait::enter(),
        SyscallType::UnlinkMkdirAt => unlink_mkdir_at::enter(),
        _ => Ok(()),
    }
}
