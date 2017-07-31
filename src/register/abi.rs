
/// Specify the ABI registers (syscall argument passing, stack pointer).
/// See sysdeps/unix/sysv/linux/${ARCH}/syscall.S from the GNU C Library.
#[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
#[macro_use]
pub mod regs_offset {
    macro_rules! get_reg {
        ($regs:ident, SysArgNum)    => ($regs.orig_rax);
        ($regs:ident, SysArg1)      => ($regs.rdi);
        ($regs:ident, SysArg2)      => ($regs.rsi);
        ($regs:ident, SysArg3)      => ($regs.rdx);
        ($regs:ident, SysArg4)      => ($regs.r10);
        ($regs:ident, SysArg5)      => ($regs.r8);
        ($regs:ident, SysArg6)      => ($regs.r9);
        ($regs:ident, SysArgResult) => ($regs.rax);
        ($regs:ident, StackPointer) => ($regs.rsp);
        ($regs:ident, InstrPointer) => ($regs.rip);
        ($regs:ident, RtldFini)     => ($regs.rdx);
        ($regs:ident, StateFlags)   => ($regs.eflags);
        ($regs:ident, UserArg1)     => ($regs.rdi);
    }
}

#[cfg(all(target_os = "linux", any(target_arch = "x86")))]
#[macro_use]
pub mod regs_offset {
    macro_rules! get_reg {
        ($regs:ident, SysArgNum)    => ($regs.orig_eax);
        ($regs:ident, SysArg1)      => ($regs.ebx);
        ($regs:ident, SysArg2)      => ($regs.ecx);
        ($regs:ident, SysArg3)      => ($regs.edx);
        ($regs:ident, SysArg4)      => ($regs.esi);
        ($regs:ident, SysArg5)      => ($regs.edi);
        ($regs:ident, SysArg6)      => ($regs.ebp);
        ($regs:ident, SysArgResult) => ($regs.eax);
        ($regs:ident, StackPointer) => ($regs.esp);
        ($regs:ident, InstrPointer) => ($regs.eip);
        ($regs:ident, RtldFini)     => ($regs.edx);
        ($regs:ident, StateFlags)   => ($regs.eflags);
        ($regs:ident, UserArg1)     => ($regs.eax);
    }
}
