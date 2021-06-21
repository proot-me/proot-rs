use sc::nr::*;

/// Used to organise system call numbers into an easily-matchable enumeration.
/// It's easier and cleaner to use cfg conditions here rather than in the huge
/// match in `translate_syscall_enter` and `translate_syscall_exit`.
#[derive(Debug, PartialEq)]
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

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn syscall_group_from_sysnum(sysnum: usize) -> SyscallGroup {
    match sysnum {
        EXECVE                                      => SyscallGroup::Execve,
        PTRACE                                      => SyscallGroup::Ptrace,
        WAIT4 /*| WAITPID*/                         => SyscallGroup::Wait,
        BRK                                         => SyscallGroup::Brk,
        GETCWD                                      => SyscallGroup::GetCwd,
        FCHDIR | CHDIR                              => SyscallGroup::Chdir,
        BIND | CONNECT                              => SyscallGroup::BindConnect,
        ACCEPT | ACCEPT4                            => SyscallGroup::Accept,
        GETSOCKNAME | GETPEERNAME                   => SyscallGroup::GetSockOrPeerName,
        /* SOCKETCALL => SyscallGroup::SocketCall, */
        // int syscall(const char *pathname, ...)
        ACCESS | ACCT | CHMOD | CHOWN /*| CHOWN32*/
            | CHROOT | GETXATTR | LISTXATTR | MKNOD
            | /*OLDSTAT |*/ CREAT | REMOVEXATTR
            | SETXATTR | STAT /*| STAT64*/ /*| STATSFS64*/
            | SWAPOFF | SWAPON | TRUNCATE /*| TRUNCATE64*/ /*| UMOUNT*/
            | UMOUNT2 | USELIB | UTIME | UTIMES     => SyscallGroup::StandardSyscall,
        // int syscall(const char *pathname, int flags, ...)
        OPEN                                        => SyscallGroup::Open,
        // int syscall(int dirfd, const char *pathname, ... , int flags, ...)
        FCHOWNAT /*| FSTATAT64*/ | NEWFSTATAT
            | UTIMENSAT | NAME_TO_HANDLE_AT | STATX => SyscallGroup::StatAt,
        // int syscall(int dirfd, const char *pathname, ...)
        FCHMODAT | FACCESSAT | FUTIMESAT | MKNODAT  => SyscallGroup::ChmodAccessMkNodAt,
        INOTIFY_ADD_WATCH                           => SyscallGroup::InotifyAddWatch,
        LCHOWN /*| LCHOWN32*/ | LGETXATTR
            | LLISTXATTR | LREMOVEXATTR | LSETXATTR
            | LSTAT /*| LSTATE64*/ /*| OLDLSTAT*/
            | UNLINK | RMDIR | MKDIR                => SyscallGroup::DirLinkAttr,
        PIVOT_ROOT                                  => SyscallGroup::PivotRoot,
        LINKAT                                      => SyscallGroup::LinkAt,
        MOUNT                                       => SyscallGroup::Mount,
        OPENAT                                      => SyscallGroup::OpenAt,
        READLINK                                    => SyscallGroup::ReadLink,
        READLINKAT                                  => SyscallGroup::ReadLinkAt,
        UNLINKAT | MKDIRAT                          => SyscallGroup::UnlinkMkdirAt,
        LINK                                        => SyscallGroup::Link,
        RENAME                                      => SyscallGroup::Rename,
        RENAMEAT                                    => SyscallGroup::RenameAt,
        SYMLINK                                     => SyscallGroup::SymLink,
        SYMLINKAT                                   => SyscallGroup::SymLinkAt,
        UNAME                                       => SyscallGroup::Uname,
        _                                           => SyscallGroup::Ignored,
    }
}
