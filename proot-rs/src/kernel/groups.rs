/// Used to organise system call numbers into an easily-matchable enumeration.
/// It's easier and cleaner to use cfg conditions here rather than in the huge
/// match in `translate_syscall_enter` and `translate_syscall_exit`.
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum SyscallGroup {
    Ignored = 0,
    Execve,
    Ptrace,
    Wait,
    Brk,
    GetCwd,
    Chdir,
    BindConnect,
    Accept,
    GetSockOrPeerName,
    #[allow(dead_code)]
    SocketCall,
    StandardSyscall, // syscalls that only require their path arguments to be translated
    Open,
    StatAt,
    ChmodAccessMkNodAt,
    InotifyAddWatch,
    DirLinkAttr,
    PivotRoot,
    LinkAt,
    Mount,
    OpenAt,
    Link,
    ReadLink,
    ReadLinkAt,
    Rename,
    RenameAt,
    SymLink,
    SymLinkAt,
    Uname,
    UnlinkMkdirAt,
}

// TODO: We also need to consider the unshare() system call. For example,
// the `CLONE_FS` flag may cause errors in our simulation of tracee's `cwd`
// field.

// TODO: modify the result of getdents64() so that we can handle binded entries.

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn syscall_group_from_sysnum(sysnum: usize) -> SyscallGroup {
    match sysnum {
        sc::nr::EXECVE => SyscallGroup::Execve,
        sc::nr::PTRACE => SyscallGroup::Ptrace,
        sc::nr::WAIT4 => SyscallGroup::Wait,
        #[cfg(any(target_arch = "x86"))]
        sc::nr::WAITPID => SyscallGroup::Wait,
        sc::nr::BRK => SyscallGroup::Brk,
        sc::nr::GETCWD => SyscallGroup::GetCwd,
        sc::nr::FCHDIR | sc::nr::CHDIR => SyscallGroup::Chdir,
        sc::nr::BIND | sc::nr::CONNECT => SyscallGroup::BindConnect,
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        sc::nr::ACCEPT => SyscallGroup::Accept,
        sc::nr::ACCEPT4 => SyscallGroup::Accept,
        sc::nr::GETSOCKNAME | sc::nr::GETPEERNAME => SyscallGroup::GetSockOrPeerName,
        #[cfg(any(target_arch = "x86"))]
        sc::nr::SOCKETCALL => SyscallGroup::SocketCall,

        // int syscall(const char *pathname, ...) follow symlink
        sc::nr::ACCT
        | sc::nr::CHROOT
        | sc::nr::GETXATTR
        | sc::nr::LISTXATTR
        | sc::nr::REMOVEXATTR
        | sc::nr::SETXATTR
        | sc::nr::SWAPOFF
        | sc::nr::SWAPON
        | sc::nr::TRUNCATE
        | sc::nr::UMOUNT2 => SyscallGroup::StandardSyscall,
        #[cfg(any(target_arch = "x86"))]
        sc::nr::OLDSTAT | sc::nr::UMOUNT => SyscallGroup::StandardSyscall,
        #[cfg(any(target_arch = "x86", target_arch = "arm"))]
        sc::nr::CHOWN32 | sc::nr::STAT64 | sc::nr::STATFS64 | sc::nr::TRUNCATE64 => {
            SyscallGroup::StandardSyscall
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        sc::nr::UTIME => SyscallGroup::StandardSyscall,
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm"))]
        sc::nr::ACCESS
        | sc::nr::CHMOD
        | sc::nr::CHOWN
        | sc::nr::MKNOD
        | sc::nr::CREAT
        | sc::nr::STAT
        | sc::nr::USELIB
        | sc::nr::UTIMES => SyscallGroup::StandardSyscall,

        // int syscall(const char *pathname, int flags, ...)
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm"))]
        sc::nr::OPEN => SyscallGroup::Open,

        // int syscall(int dirfd, const char *pathname, ... , int flags, ...)
        sc::nr::FCHOWNAT | sc::nr::UTIMENSAT | sc::nr::NAME_TO_HANDLE_AT | sc::nr::STATX => {
            SyscallGroup::StatAt
        }
        #[cfg(any(target_arch = "x86", target_arch = "arm"))]
        sc::nr::FSTATAT64 => SyscallGroup::StatAt,
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        sc::nr::NEWFSTATAT => SyscallGroup::StatAt,

        // int syscall(int dirfd, const char *pathname, ...)
        sc::nr::FCHMODAT | sc::nr::FACCESSAT | sc::nr::MKNODAT => SyscallGroup::ChmodAccessMkNodAt,
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm"))]
        sc::nr::FUTIMESAT => SyscallGroup::ChmodAccessMkNodAt,

        sc::nr::INOTIFY_ADD_WATCH => SyscallGroup::InotifyAddWatch,

        // int syscall(const char *pathname, ...) not follow symlink
        sc::nr::LGETXATTR | sc::nr::LLISTXATTR | sc::nr::LREMOVEXATTR | sc::nr::LSETXATTR => {
            SyscallGroup::DirLinkAttr
        }
        #[cfg(any(target_arch = "x86"))]
        sc::nr::OLDLSTAT => SyscallGroup::DirLinkAttr,
        #[cfg(any(target_arch = "x86", target_arch = "arm"))]
        sc::nr::LCHOWN32 | sc::nr::LSTAT64 => SyscallGroup::DirLinkAttr,
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm"))]
        sc::nr::LCHOWN | sc::nr::LSTAT | sc::nr::UNLINK | sc::nr::RMDIR | sc::nr::MKDIR => {
            SyscallGroup::DirLinkAttr
        }

        sc::nr::PIVOT_ROOT => SyscallGroup::PivotRoot,
        sc::nr::LINKAT => SyscallGroup::LinkAt,
        sc::nr::MOUNT => SyscallGroup::Mount,
        sc::nr::OPENAT => SyscallGroup::OpenAt,
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm"))]
        sc::nr::READLINK => SyscallGroup::ReadLink,
        sc::nr::READLINKAT => SyscallGroup::ReadLinkAt,
        sc::nr::UNLINKAT | sc::nr::MKDIRAT => SyscallGroup::UnlinkMkdirAt,
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm"))]
        sc::nr::LINK => SyscallGroup::Link,
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm"))]
        sc::nr::RENAME => SyscallGroup::Rename,
        sc::nr::RENAMEAT => SyscallGroup::RenameAt,
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm"))]
        sc::nr::SYMLINK => SyscallGroup::SymLink,
        sc::nr::SYMLINKAT => SyscallGroup::SymLinkAt,
        sc::nr::UNAME => SyscallGroup::Uname,
        _ => SyscallGroup::Ignored,
    }
}
