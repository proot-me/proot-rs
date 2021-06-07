use crate::errors::Result;
use crate::kernel::execve;
use crate::kernel::groups::syscall_group_from_sysnum;
use crate::kernel::groups::SyscallGroup::*;
use crate::kernel::heap::*;
use crate::kernel::ptrace::*;
use crate::kernel::socket::*;
use crate::kernel::standard::*;
use crate::process::proot::InfoBag;
use crate::process::tracee::Tracee;
use crate::register::Original;

pub fn translate(info_bag: &InfoBag, tracee: &mut Tracee) -> Result<()> {
    let sys_num = tracee.regs.get_sys_num(Original);
    let sys_type = syscall_group_from_sysnum(sys_num);

    match sys_type {
        Accept => accept::enter(),
        BindConnect => bind_connect::enter(),
        Brk => brk::enter(),
        Chdir => chdir::enter(tracee),
        ChmodAccessMkNodAt => chmod_access_mknod_at::enter(tracee),
        DirLinkAttr => dir_link_attr::enter(tracee),
        Execve => execve::enter(tracee, &info_bag.loader),
        GetCwd => getcwd::enter(tracee),
        GetSockOrPeerName => get_sockorpeer_name::enter(),
        InotifyAddWatch => inotify_add_watch::enter(),
        Link => link_rename::enter(),
        LinkAt => link_at::enter(),
        Mount => mount::enter(),
        Open => open::enter(tracee),
        OpenAt => open_at::enter(tracee),
        PivotRoot => pivot_root::enter(),
        Ptrace => ptrace::enter(),
        ReadLink => dir_link_attr::enter(tracee),
        ReadLinkAt => readlink_at::enter(),
        Rename => link_rename::enter(),
        RenameAt => rename_at::enter(),
        SocketCall => socketcall::enter(),
        StandardSyscall => standard_syscall::enter(tracee),
        StatAt => stat_at::enter(),
        SymLink => sym_link::enter(),
        SymLinkAt => sym_link_at::enter(),
        Wait => wait::enter(),
        UnlinkMkdirAt => unlink_mkdir_at::enter(),
        _ => Ok(()),
    }
}
