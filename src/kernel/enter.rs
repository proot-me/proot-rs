use errors::Result;
use kernel::execve;
use kernel::groups::syscall_group_from_sysnum;
use kernel::groups::SyscallGroup::*;
use kernel::heap::*;
use kernel::ptrace::*;
use kernel::socket::*;
use kernel::standard::*;
use process::proot::InfoBag;
use process::tracee::Tracee;
use register::Current;

pub fn translate(info_bag: &InfoBag, tracee: &mut Tracee) -> Result<()> {
    let sys_type = syscall_group_from_sysnum(tracee.regs.get_sys_num(Current));

    println!(
        "enter  \t({:?}, \t{:?}) ",
        tracee.regs.get_sys_num(Current),
        sys_type
    );

    match sys_type {
        Accept => accept::enter(),
        BindConnect => bind_connect::enter(),
        Brk => brk::enter(),
        Chdir => chdir::enter(),
        ChmodAccessMkNodAt => chmod_access_mknod_at::enter(),
        DirLinkAttr => dir_link_attr::enter(),
        Execve => execve::enter(tracee, &info_bag.loader),
        GetCwd => getcwd::enter(),
        GetSockOrPeerName => get_sockorpeer_name::enter(),
        InotifyAddWatch => inotify_add_watch::enter(),
        Link => link_rename::enter(),
        LinkAt => link_at::enter(),
        Mount => mount::enter(),
        Open => open::enter(),
        OpenAt => open_at::enter(),
        PivotRoot => pivot_root::enter(),
        Ptrace => ptrace::enter(),
        ReadLink => standard_syscall::enter(),
        ReadLinkAt => readlink_at::enter(),
        Rename => link_rename::enter(),
        RenameAt => rename_at::enter(),
        SocketCall => socketcall::enter(),
        StandardSyscall => standard_syscall::enter(),
        StatAt => stat_at::enter(),
        SymLink => sym_link::enter(),
        SymLinkAt => sym_link_at::enter(),
        Wait => wait::enter(),
        UnlinkMkdirAt => unlink_mkdir_at::enter(),
        _ => Ok(()),
    }
}
