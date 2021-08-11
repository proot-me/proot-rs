/// Specify the ABI registers (syscall argument passing, stack pointer).
/// See sysdeps/unix/sysv/linux/${ARCH}/syscall.S from the GNU C Library.
#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    any(target_arch = "x86_64")
))]
#[macro_use]
pub mod regs_offset {
    macro_rules! get_reg {
        ($regs:expr, SysNum) => {
            $regs.0.orig_rax
        };
        ($regs:expr, SysArg1) => {
            $regs.0.rdi
        };
        ($regs:expr, SysArg2) => {
            $regs.0.rsi
        };
        ($regs:expr, SysArg3) => {
            $regs.0.rdx
        };
        ($regs:expr, SysArg4) => {
            $regs.0.r10
        };
        ($regs:expr, SysArg5) => {
            $regs.0.r8
        };
        ($regs:expr, SysArg6) => {
            $regs.0.r9
        };
        ($regs:expr, SysResult) => {
            $regs.0.rax
        };
        ($regs:expr, StackPointer) => {
            $regs.0.rsp
        };
        ($regs:expr, InstrPointer) => {
            $regs.0.rip
        };
        ($regs:expr, RtldFini) => {
            $regs.0.rdx
        };
        ($regs:expr, StateFlags) => {
            $regs.0.eflags
        };
        ($regs:expr, UserArg1) => {
            $regs.0.rdi
        };
    }
}

#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    any(target_arch = "x86")
))]
#[macro_use]
pub mod regs_offset {
    macro_rules! get_reg {
        ($regs:expr, SysNum) => {
            $regs.0.orig_eax
        };
        ($regs:expr, SysArg1) => {
            $regs.0.ebx
        };
        ($regs:expr, SysArg2) => {
            $regs.0.ecx
        };
        ($regs:expr, SysArg3) => {
            $regs.0.edx
        };
        ($regs:expr, SysArg4) => {
            $regs.0.esi
        };
        ($regs:expr, SysArg5) => {
            $regs.0.edi
        };
        ($regs:expr, SysArg6) => {
            $regs.0.ebp
        };
        ($regs:expr, SysResult) => {
            $regs.0.eax
        };
        ($regs:expr, StackPointer) => {
            $regs.0.esp
        };
        ($regs:expr, InstrPointer) => {
            $regs.0.eip
        };
        ($regs:expr, RtldFini) => {
            $regs.0.edx
        };
        ($regs:expr, StateFlags) => {
            $regs.0.eflags
        };
        ($regs:expr, UserArg1) => {
            $regs.0.eax
        };
    }
}

/// https://chromium.googlesource.com/chromiumos/docs/+/master/constants/syscalls.md#calling-conventions
#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    any(target_arch = "arm")
))]
#[macro_use]
pub mod regs_offset {
    macro_rules! get_reg {
        ($regs:expr, SysNum) => {
            $regs.0[7]
        };
        ($regs:expr, SysArg1) => {
            $regs.0[0]
        };
        ($regs:expr, SysArg2) => {
            $regs.0[1]
        };
        ($regs:expr, SysArg3) => {
            $regs.0[2]
        };
        ($regs:expr, SysArg4) => {
            $regs.0[3]
        };
        ($regs:expr, SysArg5) => {
            $regs.0[4]
        };
        ($regs:expr, SysArg6) => {
            $regs.0[5]
        };
        ($regs:expr, SysResult) => {
            $regs.0[0]
        };
        ($regs:expr, StackPointer) => {
            $regs.0[13]
        };
        ($regs:expr, InstrPointer) => {
            $regs.0[15]
        };
    }
}

/// https://chromium.googlesource.com/chromiumos/docs/+/master/constants/syscalls.md#calling-conventions
#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    any(target_arch = "aarch64")
))]
#[macro_use]
pub mod regs_offset {
    macro_rules! get_reg {
        ($regs:expr, SysNum) => {
            $regs.regs[8]
        };
        ($regs:expr, SysArg1) => {
            $regs.regs[0]
        };
        ($regs:expr, SysArg2) => {
            $regs.regs[1]
        };
        ($regs:expr, SysArg3) => {
            $regs.regs[2]
        };
        ($regs:expr, SysArg4) => {
            $regs.regs[3]
        };
        ($regs:expr, SysArg5) => {
            $regs.regs[4]
        };
        ($regs:expr, SysArg6) => {
            $regs.regs[5]
        };
        ($regs:expr, SysResult) => {
            $regs.regs[0]
        };
        ($regs:expr, StackPointer) => {
            $regs.sp
        };
        ($regs:expr, InstrPointer) => {
            $regs.pc
        };
    }
}
