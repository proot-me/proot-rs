
pub mod ptrace {
    pub mod ptrace_events {
        use nix::sys::ptrace::ptrace::*;
        use nix::sys::ioctl::libc::{c_int, SIGTRAP, SIGSTOP};
        pub type PTraceSignalEvent = c_int;

        pub const RAW_SIGTRAP_SIGNAL: PTraceSignalEvent = SIGTRAP;
        pub const NORMAL_SIGTRAP_SIGNAL: PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | 0x80;
        pub const SECCOMP_SIGNAL: PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_SECCOMP << 8;
        //TODO: pub const SECCOMP2_SIGNAL:  PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_SECCOMP2 << 8;
        pub const VFORK_SIGNAL: PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_VFORK << 8;
        pub const VFORK_DONE_SIGNAL: PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_VFORK_DONE << 8;
        pub const FORK_SIGNAL: PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_FORK << 8;
        pub const CLONE_SIGNAL: PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_CLONE << 8;
        pub const EXEC_SIGNAL: PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_EXEC << 8;
        // unreachable pattern?
        // pub const EXIT_SIGNAL: PTraceSignalEvent = RAW_SIGTRAP_SIGNAL | PTRACE_EVENT_EXIT << 8;
        pub const SIGSTOP_SIGNAL: PTraceSignalEvent = SIGSTOP;
    }
}

pub mod cli {
    pub const DEFAULT_ROOTFS: &'static str = "/";
    pub const DEFAULT_CWD: &'static str = ".";
}