use std::collections::HashMap;
use std::fmt::Display;

use crate::process::tracee::Tracee;
use crate::register::Original;
use crate::register::RegVersion;
use crate::register::{
    StackPointer, SysArg, SysArg1, SysArg2, SysArg3, SysArg4, SysArg5, SysArg6, SysResult,
};

lazy_static! {
    static ref SYSNUM_TO_SYSCALL_NAME: HashMap<usize, &'static str> = [
        (sc::nr::_SYSCTL, "_sysctl"),
        (sc::nr::ACCEPT, "accept"),
        (sc::nr::ACCEPT4, "accept4"),
        (sc::nr::ACCESS, "access"),
        (sc::nr::ACCT, "acct"),
        (sc::nr::ADD_KEY, "add_key"),
        (sc::nr::ADJTIMEX, "adjtimex"),
        (sc::nr::AFS_SYSCALL, "afs_syscall"),
        (sc::nr::ALARM, "alarm"),
        (sc::nr::ARCH_PRCTL, "arch_prctl"),
        (sc::nr::BIND, "bind"),
        (sc::nr::BPF, "bpf"),
        (sc::nr::BRK, "brk"),
        (sc::nr::CAPGET, "capget"),
        (sc::nr::CAPSET, "capset"),
        (sc::nr::CHDIR, "chdir"),
        (sc::nr::CHMOD, "chmod"),
        (sc::nr::CHOWN, "chown"),
        (sc::nr::CHROOT, "chroot"),
        (sc::nr::CLOCK_ADJTIME, "clock_adjtime"),
        (sc::nr::CLOCK_GETRES, "clock_getres"),
        (sc::nr::CLOCK_GETTIME, "clock_gettime"),
        (sc::nr::CLOCK_NANOSLEEP, "clock_nanosleep"),
        (sc::nr::CLOCK_SETTIME, "clock_settime"),
        (sc::nr::CLONE, "clone"),
        (sc::nr::CLOSE, "close"),
        (sc::nr::CONNECT, "connect"),
        (sc::nr::COPY_FILE_RANGE, "copy_file_range"),
        (sc::nr::CREAT, "creat"),
        (sc::nr::CREATE_MODULE, "create_module"),
        (sc::nr::DELETE_MODULE, "delete_module"),
        (sc::nr::DUP, "dup"),
        (sc::nr::DUP2, "dup2"),
        (sc::nr::DUP3, "dup3"),
        (sc::nr::EPOLL_CREATE, "epoll_create"),
        (sc::nr::EPOLL_CREATE1, "epoll_create1"),
        (sc::nr::EPOLL_CTL, "epoll_ctl"),
        (sc::nr::EPOLL_CTL_OLD, "epoll_ctl_old"),
        (sc::nr::EPOLL_PWAIT, "epoll_pwait"),
        (sc::nr::EPOLL_WAIT, "epoll_wait"),
        (sc::nr::EPOLL_WAIT_OLD, "epoll_wait_old"),
        (sc::nr::EVENTFD, "eventfd"),
        (sc::nr::EVENTFD2, "eventfd2"),
        (sc::nr::EXECVE, "execve"),
        (sc::nr::EXECVEAT, "execveat"),
        (sc::nr::EXIT, "exit"),
        (sc::nr::EXIT_GROUP, "exit_group"),
        (sc::nr::FACCESSAT, "faccessat"),
        (sc::nr::FADVISE64, "fadvise64"),
        (sc::nr::FALLOCATE, "fallocate"),
        (sc::nr::FANOTIFY_INIT, "fanotify_init"),
        (sc::nr::FANOTIFY_MARK, "fanotify_mark"),
        (sc::nr::FCHDIR, "fchdir"),
        (sc::nr::FCHMOD, "fchmod"),
        (sc::nr::FCHMODAT, "fchmodat"),
        (sc::nr::FCHOWN, "fchown"),
        (sc::nr::FCHOWNAT, "fchownat"),
        (sc::nr::FCNTL, "fcntl"),
        (sc::nr::FDATASYNC, "fdatasync"),
        (sc::nr::FGETXATTR, "fgetxattr"),
        (sc::nr::FINIT_MODULE, "finit_module"),
        (sc::nr::FLISTXATTR, "flistxattr"),
        (sc::nr::FLOCK, "flock"),
        (sc::nr::FORK, "fork"),
        (sc::nr::FREMOVEXATTR, "fremovexattr"),
        (sc::nr::FSETXATTR, "fsetxattr"),
        (sc::nr::FSTAT, "fstat"),
        (sc::nr::FSTATFS, "fstatfs"),
        (sc::nr::FSYNC, "fsync"),
        (sc::nr::FTRUNCATE, "ftruncate"),
        (sc::nr::FUTEX, "futex"),
        (sc::nr::FUTIMESAT, "futimesat"),
        (sc::nr::GET_KERNEL_SYMS, "get_kernel_syms"),
        (sc::nr::GET_MEMPOLICY, "get_mempolicy"),
        (sc::nr::GET_ROBUST_LIST, "get_robust_list"),
        (sc::nr::GET_THREAD_AREA, "get_thread_area"),
        (sc::nr::GETCPU, "getcpu"),
        (sc::nr::GETCWD, "getcwd"),
        (sc::nr::GETDENTS, "getdents"),
        (sc::nr::GETDENTS64, "getdents64"),
        (sc::nr::GETEGID, "getegid"),
        (sc::nr::GETEUID, "geteuid"),
        (sc::nr::GETGID, "getgid"),
        (sc::nr::GETGROUPS, "getgroups"),
        (sc::nr::GETITIMER, "getitimer"),
        (sc::nr::GETPEERNAME, "getpeername"),
        (sc::nr::GETPGID, "getpgid"),
        (sc::nr::GETPGRP, "getpgrp"),
        (sc::nr::GETPID, "getpid"),
        (sc::nr::GETPMSG, "getpmsg"),
        (sc::nr::GETPPID, "getppid"),
        (sc::nr::GETPRIORITY, "getpriority"),
        (sc::nr::GETRANDOM, "getrandom"),
        (sc::nr::GETRESGID, "getresgid"),
        (sc::nr::GETRESUID, "getresuid"),
        (sc::nr::GETRLIMIT, "getrlimit"),
        (sc::nr::GETRUSAGE, "getrusage"),
        (sc::nr::GETSID, "getsid"),
        (sc::nr::GETSOCKNAME, "getsockname"),
        (sc::nr::GETSOCKOPT, "getsockopt"),
        (sc::nr::GETTID, "gettid"),
        (sc::nr::GETTIMEOFDAY, "gettimeofday"),
        (sc::nr::GETUID, "getuid"),
        (sc::nr::GETXATTR, "getxattr"),
        (sc::nr::INIT_MODULE, "init_module"),
        (sc::nr::INOTIFY_ADD_WATCH, "inotify_add_watch"),
        (sc::nr::INOTIFY_INIT, "inotify_init"),
        (sc::nr::INOTIFY_INIT1, "inotify_init1"),
        (sc::nr::INOTIFY_RM_WATCH, "inotify_rm_watch"),
        (sc::nr::IO_CANCEL, "io_cancel"),
        (sc::nr::IO_DESTROY, "io_destroy"),
        (sc::nr::IO_GETEVENTS, "io_getevents"),
        (sc::nr::IO_SETUP, "io_setup"),
        (sc::nr::IO_SUBMIT, "io_submit"),
        (sc::nr::IOCTL, "ioctl"),
        (sc::nr::IOPERM, "ioperm"),
        (sc::nr::IOPL, "iopl"),
        (sc::nr::IOPRIO_GET, "ioprio_get"),
        (sc::nr::IOPRIO_SET, "ioprio_set"),
        (sc::nr::KCMP, "kcmp"),
        (sc::nr::KEXEC_FILE_LOAD, "kexec_file_load"),
        (sc::nr::KEXEC_LOAD, "kexec_load"),
        (sc::nr::KEYCTL, "keyctl"),
        (sc::nr::KILL, "kill"),
        (sc::nr::LCHOWN, "lchown"),
        (sc::nr::LGETXATTR, "lgetxattr"),
        (sc::nr::LINK, "link"),
        (sc::nr::LINKAT, "linkat"),
        (sc::nr::LISTEN, "listen"),
        (sc::nr::LISTXATTR, "listxattr"),
        (sc::nr::LLISTXATTR, "llistxattr"),
        (sc::nr::LOOKUP_DCOOKIE, "lookup_dcookie"),
        (sc::nr::LREMOVEXATTR, "lremovexattr"),
        (sc::nr::LSEEK, "lseek"),
        (sc::nr::LSETXATTR, "lsetxattr"),
        (sc::nr::LSTAT, "lstat"),
        (sc::nr::MADVISE, "madvise"),
        (sc::nr::MBIND, "mbind"),
        (sc::nr::MEMBARRIER, "membarrier"),
        (sc::nr::MEMFD_CREATE, "memfd_create"),
        (sc::nr::MIGRATE_PAGES, "migrate_pages"),
        (sc::nr::MINCORE, "mincore"),
        (sc::nr::MKDIR, "mkdir"),
        (sc::nr::MKDIRAT, "mkdirat"),
        (sc::nr::MKNOD, "mknod"),
        (sc::nr::MKNODAT, "mknodat"),
        (sc::nr::MLOCK, "mlock"),
        (sc::nr::MLOCK2, "mlock2"),
        (sc::nr::MLOCKALL, "mlockall"),
        (sc::nr::MMAP, "mmap"),
        (sc::nr::MODIFY_LDT, "modify_ldt"),
        (sc::nr::MOUNT, "mount"),
        (sc::nr::MOVE_PAGES, "move_pages"),
        (sc::nr::MPROTECT, "mprotect"),
        (sc::nr::MQ_GETSETATTR, "mq_getsetattr"),
        (sc::nr::MQ_NOTIFY, "mq_notify"),
        (sc::nr::MQ_OPEN, "mq_open"),
        (sc::nr::MQ_TIMEDRECEIVE, "mq_timedreceive"),
        (sc::nr::MQ_TIMEDSEND, "mq_timedsend"),
        (sc::nr::MQ_UNLINK, "mq_unlink"),
        (sc::nr::MREMAP, "mremap"),
        (sc::nr::MSGCTL, "msgctl"),
        (sc::nr::MSGGET, "msgget"),
        (sc::nr::MSGRCV, "msgrcv"),
        (sc::nr::MSGSND, "msgsnd"),
        (sc::nr::MSYNC, "msync"),
        (sc::nr::MUNLOCK, "munlock"),
        (sc::nr::MUNLOCKALL, "munlockall"),
        (sc::nr::MUNMAP, "munmap"),
        (sc::nr::NAME_TO_HANDLE_AT, "name_to_handle_at"),
        (sc::nr::NANOSLEEP, "nanosleep"),
        (sc::nr::NEWFSTATAT, "newfstatat"),
        (sc::nr::NFSSERVCTL, "nfsservctl"),
        (sc::nr::OPEN, "open"),
        (sc::nr::OPEN_BY_HANDLE_AT, "open_by_handle_at"),
        (sc::nr::OPENAT, "openat"),
        (sc::nr::PAUSE, "pause"),
        (sc::nr::PERF_EVENT_OPEN, "perf_event_open"),
        (sc::nr::PERSONALITY, "personality"),
        (sc::nr::PIPE, "pipe"),
        (sc::nr::PIPE2, "pipe2"),
        (sc::nr::PIVOT_ROOT, "pivot_root"),
        (sc::nr::PKEY_ALLOC, "pkey_alloc"),
        (sc::nr::PKEY_FREE, "pkey_free"),
        (sc::nr::PKEY_MPROTECT, "pkey_mprotect"),
        (sc::nr::POLL, "poll"),
        (sc::nr::PPOLL, "ppoll"),
        (sc::nr::PRCTL, "prctl"),
        (sc::nr::PREAD64, "pread64"),
        (sc::nr::PREADV, "preadv"),
        (sc::nr::PREADV2, "preadv2"),
        (sc::nr::PRLIMIT64, "prlimit64"),
        (sc::nr::PROCESS_VM_READV, "process_vm_readv"),
        (sc::nr::PROCESS_VM_WRITEV, "process_vm_writev"),
        (sc::nr::PSELECT6, "pselect6"),
        (sc::nr::PTRACE, "ptrace"),
        (sc::nr::PUTPMSG, "putpmsg"),
        (sc::nr::PWRITE64, "pwrite64"),
        (sc::nr::PWRITEV, "pwritev"),
        (sc::nr::PWRITEV2, "pwritev2"),
        (sc::nr::QUERY_MODULE, "query_module"),
        (sc::nr::QUOTACTL, "quotactl"),
        (sc::nr::READ, "read"),
        (sc::nr::READAHEAD, "readahead"),
        (sc::nr::READLINK, "readlink"),
        (sc::nr::READLINKAT, "readlinkat"),
        (sc::nr::READV, "readv"),
        (sc::nr::REBOOT, "reboot"),
        (sc::nr::RECVFROM, "recvfrom"),
        (sc::nr::RECVMMSG, "recvmmsg"),
        (sc::nr::RECVMSG, "recvmsg"),
        (sc::nr::REMAP_FILE_PAGES, "remap_file_pages"),
        (sc::nr::REMOVEXATTR, "removexattr"),
        (sc::nr::RENAME, "rename"),
        (sc::nr::RENAMEAT, "renameat"),
        (sc::nr::RENAMEAT2, "renameat2"),
        (sc::nr::REQUEST_KEY, "request_key"),
        (sc::nr::RESTART_SYSCALL, "restart_syscall"),
        (sc::nr::RMDIR, "rmdir"),
        (sc::nr::RT_SIGACTION, "rt_sigaction"),
        (sc::nr::RT_SIGPENDING, "rt_sigpending"),
        (sc::nr::RT_SIGPROCMASK, "rt_sigprocmask"),
        (sc::nr::RT_SIGQUEUEINFO, "rt_sigqueueinfo"),
        (sc::nr::RT_SIGRETURN, "rt_sigreturn"),
        (sc::nr::RT_SIGSUSPEND, "rt_sigsuspend"),
        (sc::nr::RT_SIGTIMEDWAIT, "rt_sigtimedwait"),
        (sc::nr::RT_TGSIGQUEUEINFO, "rt_tgsigqueueinfo"),
        (sc::nr::SCHED_GET_PRIORITY_MAX, "sched_get_priority_max"),
        (sc::nr::SCHED_GET_PRIORITY_MIN, "sched_get_priority_min"),
        (sc::nr::SCHED_GETAFFINITY, "sched_getaffinity"),
        (sc::nr::SCHED_GETATTR, "sched_getattr"),
        (sc::nr::SCHED_GETPARAM, "sched_getparam"),
        (sc::nr::SCHED_GETSCHEDULER, "sched_getscheduler"),
        (sc::nr::SCHED_RR_GET_INTERVAL, "sched_rr_get_interval"),
        (sc::nr::SCHED_SETAFFINITY, "sched_setaffinity"),
        (sc::nr::SCHED_SETATTR, "sched_setattr"),
        (sc::nr::SCHED_SETPARAM, "sched_setparam"),
        (sc::nr::SCHED_SETSCHEDULER, "sched_setscheduler"),
        (sc::nr::SCHED_YIELD, "sched_yield"),
        (sc::nr::SECCOMP, "seccomp"),
        (sc::nr::SECURITY, "security"),
        (sc::nr::SELECT, "select"),
        (sc::nr::SEMCTL, "semctl"),
        (sc::nr::SEMGET, "semget"),
        (sc::nr::SEMOP, "semop"),
        (sc::nr::SEMTIMEDOP, "semtimedop"),
        (sc::nr::SENDFILE, "sendfile"),
        (sc::nr::SENDMMSG, "sendmmsg"),
        (sc::nr::SENDMSG, "sendmsg"),
        (sc::nr::SENDTO, "sendto"),
        (sc::nr::SET_MEMPOLICY, "set_mempolicy"),
        (sc::nr::SET_ROBUST_LIST, "set_robust_list"),
        (sc::nr::SET_THREAD_AREA, "set_thread_area"),
        (sc::nr::SET_TID_ADDRESS, "set_tid_address"),
        (sc::nr::SETDOMAINNAME, "setdomainname"),
        (sc::nr::SETFSGID, "setfsgid"),
        (sc::nr::SETFSUID, "setfsuid"),
        (sc::nr::SETGID, "setgid"),
        (sc::nr::SETGROUPS, "setgroups"),
        (sc::nr::SETHOSTNAME, "sethostname"),
        (sc::nr::SETITIMER, "setitimer"),
        (sc::nr::SETNS, "setns"),
        (sc::nr::SETPGID, "setpgid"),
        (sc::nr::SETPRIORITY, "setpriority"),
        (sc::nr::SETREGID, "setregid"),
        (sc::nr::SETRESGID, "setresgid"),
        (sc::nr::SETRESUID, "setresuid"),
        (sc::nr::SETREUID, "setreuid"),
        (sc::nr::SETRLIMIT, "setrlimit"),
        (sc::nr::SETSID, "setsid"),
        (sc::nr::SETSOCKOPT, "setsockopt"),
        (sc::nr::SETTIMEOFDAY, "settimeofday"),
        (sc::nr::SETUID, "setuid"),
        (sc::nr::SETXATTR, "setxattr"),
        (sc::nr::SHMAT, "shmat"),
        (sc::nr::SHMCTL, "shmctl"),
        (sc::nr::SHMDT, "shmdt"),
        (sc::nr::SHMGET, "shmget"),
        (sc::nr::SHUTDOWN, "shutdown"),
        (sc::nr::SIGALTSTACK, "sigaltstack"),
        (sc::nr::SIGNALFD, "signalfd"),
        (sc::nr::SIGNALFD4, "signalfd4"),
        (sc::nr::SOCKET, "socket"),
        (sc::nr::SOCKETPAIR, "socketpair"),
        (sc::nr::SPLICE, "splice"),
        (sc::nr::STAT, "stat"),
        (sc::nr::STATFS, "statfs"),
        (sc::nr::STATX, "statx"),
        (sc::nr::SWAPOFF, "swapoff"),
        (sc::nr::SWAPON, "swapon"),
        (sc::nr::SYMLINK, "symlink"),
        (sc::nr::SYMLINKAT, "symlinkat"),
        (sc::nr::SYNC, "sync"),
        (sc::nr::SYNC_FILE_RANGE, "sync_file_range"),
        (sc::nr::SYNCFS, "syncfs"),
        (sc::nr::SYSFS, "sysfs"),
        (sc::nr::SYSINFO, "sysinfo"),
        (sc::nr::SYSLOG, "syslog"),
        (sc::nr::TEE, "tee"),
        (sc::nr::TGKILL, "tgkill"),
        (sc::nr::TIME, "time"),
        (sc::nr::TIMER_CREATE, "timer_create"),
        (sc::nr::TIMER_DELETE, "timer_delete"),
        (sc::nr::TIMER_GETOVERRUN, "timer_getoverrun"),
        (sc::nr::TIMER_GETTIME, "timer_gettime"),
        (sc::nr::TIMER_SETTIME, "timer_settime"),
        (sc::nr::TIMERFD_CREATE, "timerfd_create"),
        (sc::nr::TIMERFD_GETTIME, "timerfd_gettime"),
        (sc::nr::TIMERFD_SETTIME, "timerfd_settime"),
        (sc::nr::TIMES, "times"),
        (sc::nr::TKILL, "tkill"),
        (sc::nr::TRUNCATE, "truncate"),
        (sc::nr::TUXCALL, "tuxcall"),
        (sc::nr::UMASK, "umask"),
        (sc::nr::UMOUNT2, "umount2"),
        (sc::nr::UNAME, "uname"),
        (sc::nr::UNLINK, "unlink"),
        (sc::nr::UNLINKAT, "unlinkat"),
        (sc::nr::UNSHARE, "unshare"),
        (sc::nr::USELIB, "uselib"),
        (sc::nr::USERFAULTFD, "userfaultfd"),
        (sc::nr::USTAT, "ustat"),
        (sc::nr::UTIME, "utime"),
        (sc::nr::UTIMENSAT, "utimensat"),
        (sc::nr::UTIMES, "utimes"),
        (sc::nr::VFORK, "vfork"),
        (sc::nr::VHANGUP, "vhangup"),
        (sc::nr::VMSPLICE, "vmsplice"),
        (sc::nr::VSERVER, "vserver"),
        (sc::nr::WAIT4, "wait4"),
        (sc::nr::WAITID, "waitid"),
        (sc::nr::WRITE, "write"),
        (sc::nr::WRITEV, "writev"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::WAITPID, "waitpid"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::SOCKETCALL, "socketcall"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::OLDSTAT, "oldstat"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::UMOUNT, "umount"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::OLDLSTAT, "oldlstat"),
        #[cfg(any(target_arch = "x86", target_arch = "arm"))]
        (sc::nr::CHOWN32, "chown32"),
        #[cfg(any(target_arch = "x86", target_arch = "arm"))]
        (sc::nr::STAT64, "stat64"),
        #[cfg(any(target_arch = "x86", target_arch = "arm"))]
        (sc::nr::STATFS64, "statfs64"),
        #[cfg(any(target_arch = "x86", target_arch = "arm"))]
        (sc::nr::TRUNCATE64, "truncate64"),
        #[cfg(any(target_arch = "x86", target_arch = "arm"))]
        (sc::nr::LCHOWN32, "lchown32"),
        #[cfg(any(target_arch = "x86", target_arch = "arm"))]
        (sc::nr::LSTAT64, "lstat64"),
    ]
    .iter()
    .cloned()
    .collect();
}

pub fn name_of_syscall(sysnum: usize) -> Option<&'static str> {
    SYSNUM_TO_SYSCALL_NAME.get(&sysnum).map(|s| *s)
}

pub fn print_syscall<M>(tracee: &Tracee, version: RegVersion, msg: M)
where
    M: Display,
{
    let sysnum = tracee.regs.get_sys_num(Original);

    trace!(
        "-- {} {}<{}>(0x{:x?}, 0x{:x?}, 0x{:x?}, 0x{:x?}, 0x{:x?}, 0x{:x?}) = 0x{:x?} [0x{:x?}] {}",
        tracee.pid,
        name_of_syscall(sysnum).unwrap_or("unknown"),
        sysnum,
        tracee.regs.get(version, SysArg(SysArg1)),
        tracee.regs.get(version, SysArg(SysArg2)),
        tracee.regs.get(version, SysArg(SysArg3)),
        tracee.regs.get(version, SysArg(SysArg4)),
        tracee.regs.get(version, SysArg(SysArg5)),
        tracee.regs.get(version, SysArg(SysArg6)),
        tracee.regs.get(version, SysResult),
        tracee.regs.get(version, StackPointer),
        msg,
    )
}
