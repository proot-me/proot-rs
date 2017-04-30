
pub mod ptrace {
    pub mod ptrace_events {
        use nix::sys::ptrace::ptrace::*;
        use nix::sys::ioctl::libc::{c_int, SIGTRAP, SIGSTOP};

        pub type PtraceSignalEvent = c_int;

        pub const PTRACE_S_SIGSTOP:           PtraceSignalEvent = SIGSTOP;
        pub const PTRACE_S_RAW_SIGTRAP:       PtraceSignalEvent = SIGTRAP;
        pub const PTRACE_S_NORMAL_SIGTRAP:    PtraceSignalEvent = SIGTRAP | 0x80;
        pub const PTRACE_S_VFORK:             PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_VFORK << 8;
        pub const PTRACE_S_VFORK_DONE:        PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_VFORK_DONE << 8;
        pub const PTRACE_S_FORK:              PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_FORK << 8;
        pub const PTRACE_S_CLONE:             PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_CLONE << 8;
        pub const PTRACE_S_EXEC:              PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_EXEC << 8;
        pub const PTRACE_S_SECCOMP:           PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_SECCOMP << 8;
        pub const PTRACE_S_SECCOMP2:          PtraceSignalEvent = SIGTRAP | PTRACE_EVENT_SECCOMP + 1 << 8;
        // unreachable pattern?
        // pub const EXIT_SIGNAL:               PTraceSignalEvent = SIGTRAP | PTRACE_EVENT_EXIT << 8;
    }
}

pub mod cli {
    pub const DEFAULT_ROOTFS: &'static str = "/";
    pub const DEFAULT_CWD: &'static str = ".";
}

pub mod tracee {
    #[derive(Debug)]
    pub enum TraceeStatus {
        /// Enter stage
        SysEnter,
        /// Exit stage
        SysExit
    }

    #[derive(Debug)]
    pub enum TraceeRestartMethod {
        /// Restart the tracee, without going through the exit stage
        WithoutExitStage,   // PTRACE_CONT
        /// Restart the tracee, with the exit stage
        WithExitStage       // PTRACE_SYSCALL
    }
}