use syscall::nr::*;

/// Used to organise system call numbers into an easily-matchable enumeration.
/// It's easier and cleaner to use cfg conditions here rather than in the huge match
/// in `translate_syscall_enter` and `translate_syscall_exit`.
#[derive(Debug, PartialEq)]
pub enum SyscallType {
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
    StandardSyscall, // kernel that only require their path arguments to be translated
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
pub fn syscall_type_from_sysnum(sysnum: usize) -> SyscallType {
    match sysnum {
        EXECVE                                      => SyscallType::Execve,
        PTRACE                                      => SyscallType::Ptrace,
        WAIT4 /*| WAITPID*/                         => SyscallType::Wait,
        BRK                                         => SyscallType::Brk,
        GETCWD                                      => SyscallType::GetCwd,
        FCHDIR | CHDIR                              => SyscallType::Chdir,
        BIND | CONNECT                              => SyscallType::BindConnect,
        ACCEPT | ACCEPT4                            => SyscallType::Accept,
        GETSOCKNAME | GETPEERNAME                   => SyscallType::GetSockOrPeerName,
        /* SOCKETCALL => SyscallType::SocketCall, */
        ACCESS | ACCT | CHMOD | CHOWN /*| CHOWN32*/
            | CHROOT | GETXATTR | LISTXATTR | MKNOD
            | /*OLDSTAT |*/ CREAT | REMOVEXATTR
            | SETXATTR | STAT /*| STAT64*/ /*| STATSFS64*/
            | SWAPOFF | SWAPON | TRUNCATE /*| TRUNCATE64*/ /*| UMOUNT*/
            | UMOUNT2 | USELIB | UTIME | UTIMES     => SyscallType::StandardSyscall,
        OPEN                                        => SyscallType::Open,
        FCHOWNAT /*| FSTATAT64*/ | NEWFSTATAT
            | UTIMENSAT | NAME_TO_HANDLE_AT         => SyscallType::StatAt,
        FCHMODAT | FACCESSAT | FUTIMESAT | MKNODAT  => SyscallType::ChmodAccessMkNodAt,
        INOTIFY_ADD_WATCH                           => SyscallType::InotifyAddWatch,
        LCHOWN /*| LCHOWN32*/ | LGETXATTR
            | LLISTXATTR | LREMOVEXATTR | LSETXATTR
            | LSTAT /*| LSTATE64*/ /*| OLDLSTAT*/
            | UNLINK | RMDIR | MKDIR                => SyscallType::DirLinkAttr,
        PIVOT_ROOT                                  => SyscallType::PivotRoot,
        LINKAT                                      => SyscallType::LinkAt,
        MOUNT                                       => SyscallType::Mount,
        OPENAT                                      => SyscallType::OpenAt,
        READLINK                                    => SyscallType::ReadLink,
        READLINKAT                                  => SyscallType::ReadLinkAt,
        UNLINKAT | MKDIRAT                          => SyscallType::UnlinkMkdirAt,
        LINK                                        => SyscallType::Link,
        RENAME                                      => SyscallType::Rename,
        RENAMEAT                                    => SyscallType::RenameAt,
        SYMLINK                                     => SyscallType::SymLink,
        SYMLINKAT                                   => SyscallType::SymLinkAt,
        UNAME                                       => SyscallType::Uname,
        _                                           => SyscallType::Ignored,
    }
}
