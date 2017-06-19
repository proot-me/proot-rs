
//todo: remove all this when a Nix PR related to this is merged
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