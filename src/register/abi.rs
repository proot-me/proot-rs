
/// Specify the ABI registers (syscall argument passing, stack pointer).
/// See sysdeps/unix/sysv/linux/${ARCH}/syscall.S from the GNU C Library.
#[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
#[macro_use]
pub mod regs_offset {
    macro_rules! get_reg {
        ($regs:expr, SysArgNum)    => ($regs.orig_rax);
        ($regs:expr, SysArg1)      => ($regs.rdi);
        ($regs:expr, SysArg2)      => ($regs.rsi);
        ($regs:expr, SysArg3)      => ($regs.rdx);
        ($regs:expr, SysArg4)      => ($regs.r10);
        ($regs:expr, SysArg5)      => ($regs.r8);
        ($regs:expr, SysArg6)      => ($regs.r9);
        ($regs:expr, SysArgResult) => ($regs.rax);
        ($regs:expr, StackPointer) => ($regs.rsp);
        ($regs:expr, InstrPointer) => ($regs.rip);
        ($regs:expr, RtldFini)     => ($regs.rdx);
        ($regs:expr, StateFlags)   => ($regs.eflags);
        ($regs:expr, UserArg1)     => ($regs.rdi);
    }
}

#[cfg(all(target_os = "linux", any(target_arch = "x86")))]
#[macro_use]
pub mod regs_offset {
    macro_rules! get_reg {
        ($regs:expr, SysArgNum)    => ($regs.orig_eax);
        ($regs:expr, SysArg1)      => ($regs.ebx);
        ($regs:expr, SysArg2)      => ($regs.ecx);
        ($regs:expr, SysArg3)      => ($regs.edx);
        ($regs:expr, SysArg4)      => ($regs.esi);
        ($regs:expr, SysArg5)      => ($regs.edi);
        ($regs:expr, SysArg6)      => ($regs.ebp);
        ($regs:expr, SysArgResult) => ($regs.eax);
        ($regs:expr, StackPointer) => ($regs.esp);
        ($regs:expr, InstrPointer) => ($regs.eip);
        ($regs:expr, RtldFini)     => ($regs.edx);
        ($regs:expr, StateFlags)   => ($regs.eflags);
        ($regs:expr, UserArg1)     => ($regs.eax);
    }
}
