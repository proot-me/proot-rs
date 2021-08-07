use std::collections::HashMap;
use std::fmt::Display;

use crate::process::tracee::Tracee;
use crate::register::Original;
use crate::register::RegVersion;
use crate::register::{
    StackPointer, SysArg, SysArg1, SysArg2, SysArg3, SysArg4, SysArg5, SysArg6, SysResult,
};

lazy_static! {
    // Generated from https://chromium.googlesource.com/chromiumos/docs/+/master/constants/syscalls.md#cross_arch-numbers
    #[cfg(any(target_os = "linux", target_os = "android"))]
    static ref SYSNUM_TO_SYSCALL_NAME: HashMap<usize, &'static str> = [
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::ARM_BREAKPOINT, "ARM_breakpoint"),
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::ARM_CACHEFLUSH, "ARM_cacheflush"),
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::ARM_SET_TLS, "ARM_set_tls"),
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::ARM_USR26, "ARM_usr26"),
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::ARM_USR32, "ARM_usr32"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::_LLSEEK, "_llseek"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::_NEWSELECT, "_newselect"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::_SYSCTL, "_sysctl"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::ACCEPT, "accept"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::ACCEPT4, "accept4"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::ACCESS, "access"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::ACCT, "acct"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::ADD_KEY, "add_key"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::ADJTIMEX, "adjtimex"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::AFS_SYSCALL, "afs_syscall"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::ALARM, "alarm"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::ARCH_PRCTL, "arch_prctl"),
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::ARM_FADVISE64_64, "arm_fadvise64_64"),
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::ARM_SYNC_FILE_RANGE, "arm_sync_file_range"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::BDFLUSH, "bdflush"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::BIND, "bind"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::BPF, "bpf"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::BREAK, "break"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::BRK, "brk"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CAPGET, "capget"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CAPSET, "capset"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CHDIR, "chdir"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::CHMOD, "chmod"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::CHOWN, "chown"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::CHOWN32, "chown32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CHROOT, "chroot"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CLOCK_ADJTIME, "clock_adjtime"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CLOCK_GETRES, "clock_getres"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CLOCK_GETTIME, "clock_gettime"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CLOCK_NANOSLEEP, "clock_nanosleep"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CLOCK_SETTIME, "clock_settime"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CLONE, "clone"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CLOSE, "close"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::CONNECT, "connect"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::COPY_FILE_RANGE, "copy_file_range"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::CREAT, "creat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::CREATE_MODULE, "create_module"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::DELETE_MODULE, "delete_module"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::DUP, "dup"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::DUP2, "dup2"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::DUP3, "dup3"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::EPOLL_CREATE, "epoll_create"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::EPOLL_CREATE1, "epoll_create1"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::EPOLL_CTL, "epoll_ctl"),
        #[cfg(any(target_arch = "x86_64"))]
        (sc::nr::EPOLL_CTL_OLD, "epoll_ctl_old"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::EPOLL_PWAIT, "epoll_pwait"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::EPOLL_WAIT, "epoll_wait"),
        #[cfg(any(target_arch = "x86_64"))]
        (sc::nr::EPOLL_WAIT_OLD, "epoll_wait_old"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::EVENTFD, "eventfd"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::EVENTFD2, "eventfd2"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::EXECVE, "execve"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::EXECVEAT, "execveat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::EXIT, "exit"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::EXIT_GROUP, "exit_group"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FACCESSAT, "faccessat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FADVISE64, "fadvise64"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::FADVISE64_64, "fadvise64_64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FALLOCATE, "fallocate"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FANOTIFY_INIT, "fanotify_init"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FANOTIFY_MARK, "fanotify_mark"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FCHDIR, "fchdir"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FCHMOD, "fchmod"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FCHMODAT, "fchmodat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FCHOWN, "fchown"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::FCHOWN32, "fchown32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FCHOWNAT, "fchownat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FCNTL, "fcntl"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::FCNTL64, "fcntl64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FDATASYNC, "fdatasync"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FGETXATTR, "fgetxattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FINIT_MODULE, "finit_module"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FLISTXATTR, "flistxattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FLOCK, "flock"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::FORK, "fork"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FREMOVEXATTR, "fremovexattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FSETXATTR, "fsetxattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FSTAT, "fstat"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::FSTAT64, "fstat64"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::FSTATAT64, "fstatat64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FSTATFS, "fstatfs"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::FSTATFS64, "fstatfs64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FSYNC, "fsync"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::FTIME, "ftime"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FTRUNCATE, "ftruncate"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::FTRUNCATE64, "ftruncate64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::FUTEX, "futex"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::FUTIMESAT, "futimesat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::GET_KERNEL_SYMS, "get_kernel_syms"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GET_MEMPOLICY, "get_mempolicy"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GET_ROBUST_LIST, "get_robust_list"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::GET_THREAD_AREA, "get_thread_area"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETCPU, "getcpu"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETCWD, "getcwd"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::GETDENTS, "getdents"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETDENTS64, "getdents64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETEGID, "getegid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::GETEGID32, "getegid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETEUID, "geteuid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::GETEUID32, "geteuid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETGID, "getgid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::GETGID32, "getgid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETGROUPS, "getgroups"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::GETGROUPS32, "getgroups32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETITIMER, "getitimer"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETPEERNAME, "getpeername"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETPGID, "getpgid"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::GETPGRP, "getpgrp"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETPID, "getpid"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::GETPMSG, "getpmsg"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETPPID, "getppid"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETPRIORITY, "getpriority"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETRANDOM, "getrandom"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETRESGID, "getresgid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::GETRESGID32, "getresgid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETRESUID, "getresuid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::GETRESUID32, "getresuid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETRLIMIT, "getrlimit"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETRUSAGE, "getrusage"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETSID, "getsid"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETSOCKNAME, "getsockname"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETSOCKOPT, "getsockopt"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETTID, "gettid"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETTIMEOFDAY, "gettimeofday"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETUID, "getuid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::GETUID32, "getuid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::GETXATTR, "getxattr"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::GTTY, "gtty"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::IDLE, "idle"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::INIT_MODULE, "init_module"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::INOTIFY_ADD_WATCH, "inotify_add_watch"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::INOTIFY_INIT, "inotify_init"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::INOTIFY_INIT1, "inotify_init1"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::INOTIFY_RM_WATCH, "inotify_rm_watch"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::IO_CANCEL, "io_cancel"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::IO_DESTROY, "io_destroy"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::IO_GETEVENTS, "io_getevents"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::IO_SETUP, "io_setup"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::IO_SUBMIT, "io_submit"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::IOCTL, "ioctl"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::IOPERM, "ioperm"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::IOPL, "iopl"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::IOPRIO_GET, "ioprio_get"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::IOPRIO_SET, "ioprio_set"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::IPC, "ipc"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::KCMP, "kcmp"),
        #[cfg(any(target_arch = "x86_64"))]
        (sc::nr::KEXEC_FILE_LOAD, "kexec_file_load"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::KEXEC_LOAD, "kexec_load"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::KEYCTL, "keyctl"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::KILL, "kill"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::LCHOWN, "lchown"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::LCHOWN32, "lchown32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::LGETXATTR, "lgetxattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::LINK, "link"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::LINKAT, "linkat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::LISTEN, "listen"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::LISTXATTR, "listxattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::LLISTXATTR, "llistxattr"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::LOCK, "lock"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::LOOKUP_DCOOKIE, "lookup_dcookie"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::LREMOVEXATTR, "lremovexattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::LSEEK, "lseek"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::LSETXATTR, "lsetxattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::LSTAT, "lstat"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::LSTAT64, "lstat64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MADVISE, "madvise"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MBIND, "mbind"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MEMBARRIER, "membarrier"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MEMFD_CREATE, "memfd_create"),
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MIGRATE_PAGES, "migrate_pages"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MINCORE, "mincore"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::MKDIR, "mkdir"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MKDIRAT, "mkdirat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::MKNOD, "mknod"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MKNODAT, "mknodat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MLOCK, "mlock"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MLOCK2, "mlock2"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MLOCKALL, "mlockall"),
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MMAP, "mmap"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::MMAP2, "mmap2"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::MODIFY_LDT, "modify_ldt"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MOUNT, "mount"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MOVE_PAGES, "move_pages"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MPROTECT, "mprotect"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::MPX, "mpx"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MQ_GETSETATTR, "mq_getsetattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MQ_NOTIFY, "mq_notify"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MQ_OPEN, "mq_open"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MQ_TIMEDRECEIVE, "mq_timedreceive"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MQ_TIMEDSEND, "mq_timedsend"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MQ_UNLINK, "mq_unlink"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MREMAP, "mremap"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::MSGCTL, "msgctl"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::MSGGET, "msgget"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::MSGRCV, "msgrcv"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::MSGSND, "msgsnd"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MSYNC, "msync"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MUNLOCK, "munlock"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MUNLOCKALL, "munlockall"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::MUNMAP, "munmap"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::NAME_TO_HANDLE_AT, "name_to_handle_at"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::NANOSLEEP, "nanosleep"),
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        (sc::nr::NEWFSTATAT, "newfstatat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::NFSSERVCTL, "nfsservctl"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::NICE, "nice"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::OLDFSTAT, "oldfstat"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::OLDLSTAT, "oldlstat"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::OLDOLDUNAME, "oldolduname"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::OLDSTAT, "oldstat"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::OLDUNAME, "olduname"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::OPEN, "open"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::OPEN_BY_HANDLE_AT, "open_by_handle_at"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::OPENAT, "openat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::PAUSE, "pause"),
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::PCICONFIG_IOBASE, "pciconfig_iobase"),
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::PCICONFIG_READ, "pciconfig_read"),
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::PCICONFIG_WRITE, "pciconfig_write"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PERF_EVENT_OPEN, "perf_event_open"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PERSONALITY, "personality"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::PIPE, "pipe"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PIPE2, "pipe2"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PIVOT_ROOT, "pivot_root"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PKEY_ALLOC, "pkey_alloc"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PKEY_FREE, "pkey_free"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PKEY_MPROTECT, "pkey_mprotect"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::POLL, "poll"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PPOLL, "ppoll"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PRCTL, "prctl"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PREAD64, "pread64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PREADV, "preadv"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PREADV2, "preadv2"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PRLIMIT64, "prlimit64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PROCESS_VM_READV, "process_vm_readv"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PROCESS_VM_WRITEV, "process_vm_writev"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::PROF, "prof"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::PROFIL, "profil"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PSELECT6, "pselect6"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PTRACE, "ptrace"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::PUTPMSG, "putpmsg"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PWRITE64, "pwrite64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PWRITEV, "pwritev"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::PWRITEV2, "pwritev2"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::QUERY_MODULE, "query_module"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::QUOTACTL, "quotactl"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::READ, "read"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::READAHEAD, "readahead"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::READDIR, "readdir"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::READLINK, "readlink"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::READLINKAT, "readlinkat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::READV, "readv"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::REBOOT, "reboot"),
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::RECV, "recv"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RECVFROM, "recvfrom"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RECVMMSG, "recvmmsg"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RECVMSG, "recvmsg"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::REMAP_FILE_PAGES, "remap_file_pages"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::REMOVEXATTR, "removexattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::RENAME, "rename"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RENAMEAT, "renameat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RENAMEAT2, "renameat2"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::REQUEST_KEY, "request_key"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RESTART_SYSCALL, "restart_syscall"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::RMDIR, "rmdir"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RT_SIGACTION, "rt_sigaction"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RT_SIGPENDING, "rt_sigpending"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RT_SIGPROCMASK, "rt_sigprocmask"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RT_SIGQUEUEINFO, "rt_sigqueueinfo"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RT_SIGRETURN, "rt_sigreturn"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RT_SIGSUSPEND, "rt_sigsuspend"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RT_SIGTIMEDWAIT, "rt_sigtimedwait"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::RT_TGSIGQUEUEINFO, "rt_tgsigqueueinfo"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_GET_PRIORITY_MAX, "sched_get_priority_max"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_GET_PRIORITY_MIN, "sched_get_priority_min"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_GETAFFINITY, "sched_getaffinity"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_GETATTR, "sched_getattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_GETPARAM, "sched_getparam"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_GETSCHEDULER, "sched_getscheduler"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_RR_GET_INTERVAL, "sched_rr_get_interval"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_SETAFFINITY, "sched_setaffinity"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_SETATTR, "sched_setattr"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_SETPARAM, "sched_setparam"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_SETSCHEDULER, "sched_setscheduler"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SCHED_YIELD, "sched_yield"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SECCOMP, "seccomp"),
        #[cfg(any(target_arch = "x86_64"))]
        (sc::nr::SECURITY, "security"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::SELECT, "select"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::SEMCTL, "semctl"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::SEMGET, "semget"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::SEMOP, "semop"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::SEMTIMEDOP, "semtimedop"),
        #[cfg(any(target_arch = "arm"))]
        (sc::nr::SEND, "send"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SENDFILE, "sendfile"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SENDFILE64, "sendfile64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SENDMMSG, "sendmmsg"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SENDMSG, "sendmsg"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SENDTO, "sendto"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SET_MEMPOLICY, "set_mempolicy"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SET_ROBUST_LIST, "set_robust_list"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::SET_THREAD_AREA, "set_thread_area"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SET_TID_ADDRESS, "set_tid_address"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETDOMAINNAME, "setdomainname"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETFSGID, "setfsgid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SETFSGID32, "setfsgid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETFSUID, "setfsuid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SETFSUID32, "setfsuid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETGID, "setgid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SETGID32, "setgid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETGROUPS, "setgroups"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SETGROUPS32, "setgroups32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETHOSTNAME, "sethostname"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETITIMER, "setitimer"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETNS, "setns"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETPGID, "setpgid"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETPRIORITY, "setpriority"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETREGID, "setregid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SETREGID32, "setregid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETRESGID, "setresgid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SETRESGID32, "setresgid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETRESUID, "setresuid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SETRESUID32, "setresuid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETREUID, "setreuid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SETREUID32, "setreuid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETRLIMIT, "setrlimit"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETSID, "setsid"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETSOCKOPT, "setsockopt"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETTIMEOFDAY, "settimeofday"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETUID, "setuid"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SETUID32, "setuid32"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SETXATTR, "setxattr"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::SGETMASK, "sgetmask"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::SHMAT, "shmat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::SHMCTL, "shmctl"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::SHMDT, "shmdt"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64"))]
        (sc::nr::SHMGET, "shmget"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SHUTDOWN, "shutdown"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SIGACTION, "sigaction"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SIGALTSTACK, "sigaltstack"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::SIGNAL, "signal"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SIGNALFD, "signalfd"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SIGNALFD4, "signalfd4"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SIGPENDING, "sigpending"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SIGPROCMASK, "sigprocmask"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SIGRETURN, "sigreturn"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SIGSUSPEND, "sigsuspend"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SOCKET, "socket"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::SOCKETCALL, "socketcall"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SOCKETPAIR, "socketpair"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SPLICE, "splice"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::SSETMASK, "ssetmask"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::STAT, "stat"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::STAT64, "stat64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::STATFS, "statfs"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::STATFS64, "statfs64"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::STATX, "statx"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::STIME, "stime"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::STTY, "stty"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SWAPOFF, "swapoff"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SWAPON, "swapon"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SYMLINK, "symlink"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SYMLINKAT, "symlinkat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SYNC, "sync"),
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SYNC_FILE_RANGE, "sync_file_range"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SYNCFS, "syncfs"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::SYSFS, "sysfs"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SYSINFO, "sysinfo"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::SYSLOG, "syslog"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TEE, "tee"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TGKILL, "tgkill"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::TIME, "time"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TIMER_CREATE, "timer_create"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TIMER_DELETE, "timer_delete"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TIMER_GETOVERRUN, "timer_getoverrun"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TIMER_GETTIME, "timer_gettime"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TIMER_SETTIME, "timer_settime"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TIMERFD_CREATE, "timerfd_create"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TIMERFD_GETTIME, "timerfd_gettime"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TIMERFD_SETTIME, "timerfd_settime"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TIMES, "times"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TKILL, "tkill"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::TRUNCATE, "truncate"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::TRUNCATE64, "truncate64"),
        #[cfg(any(target_arch = "x86_64"))]
        (sc::nr::TUXCALL, "tuxcall"),
        #[cfg(any(target_arch = "arm", target_arch = "x86"))]
        (sc::nr::UGETRLIMIT, "ugetrlimit"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::ULIMIT, "ulimit"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::UMASK, "umask"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::UMOUNT, "umount"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::UMOUNT2, "umount2"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::UNAME, "uname"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::UNLINK, "unlink"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::UNLINKAT, "unlinkat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::UNSHARE, "unshare"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::USELIB, "uselib"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::USERFAULTFD, "userfaultfd"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::USTAT, "ustat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        (sc::nr::UTIME, "utime"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::UTIMENSAT, "utimensat"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::UTIMES, "utimes"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::VFORK, "vfork"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::VHANGUP, "vhangup"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::VM86, "vm86"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::VM86OLD, "vm86old"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::VMSPLICE, "vmsplice"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "x86"))]
        (sc::nr::VSERVER, "vserver"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::WAIT4, "wait4"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::WAITID, "waitid"),
        #[cfg(any(target_arch = "x86"))]
        (sc::nr::WAITPID, "waitpid"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::WRITE, "write"),
        #[cfg(any(target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64", target_arch = "x86"))]
        (sc::nr::WRITEV, "writev"),
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
