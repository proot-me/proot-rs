use std::ptr::null_mut;
use std::mem;
use libc::{c_void, user_regs_struct};
use errors::Result;
use nix::unistd::Pid;
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::PTRACE_GETREGS;

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

/// Copy all @tracee's general purpose registers into a dedicated cache.
/// Returns either `Ok(regs)` or `Err(Sys(errno))` or `Err(InvalidPath)`.
#[inline]
pub fn fetch_all_regs(pid: Pid) -> Result<user_regs_struct> {
    let mut regs: user_regs_struct = unsafe { mem::zeroed() };
    let p_regs: *mut c_void = &mut regs as *mut _ as *mut c_void;

    // Notice the ? at the end, which is the equivalent of `try!`.
    // It will return the error if there is one.
    ptrace(PTRACE_GETREGS, pid, null_mut(), p_regs)?;

    Ok(regs)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use nix::unistd::{Pid, execvp};
    use syscall::nr::NANOSLEEP;
    use utils::tests::fork_test;

    #[test]
    fn fetch_regs_should_fail_test() {
        let ret = fetch_all_regs(Pid::from_raw(-1));
        assert!(ret.is_err());
    }

    #[test]
    fn fetch_regs_test() {
        fork_test(
            // expecting a normal execution
            0,
            // parent
            |_, _| {
                // we stop on the first syscall;
                // the fact that no panic was sparked until now means that the regs were OK
                return true;
            },
            // child
            || {
                // calling the sleep function, which should call the NANOSLEEP syscall
                execvp(
                    &CString::new("sleep").unwrap(),
                    &[CString::new(".").unwrap(), CString::new("0").unwrap()],
                ).expect("failed execvp sleep");
            },
        );
    }

    #[test]
    /// Tests that `fetch_regs` works on a simple syscall;
    /// the test is a success if the NANOSLEEP syscall is detected (with its corresponding signum).
    fn fetch_regs_sysnum_sleep_test() {
        fork_test(
            // expecting a normal execution
            0,
            // parent
            |_, regs| {
                // we only stop when the NANOSLEEP syscall is detected
                return regs.sys_num == NANOSLEEP;
            },
            // child
            || {
                // calling the sleep function, which should call the NANOSLEEP syscall
                execvp(
                    &CString::new("sleep").unwrap(),
                    &[CString::new(".").unwrap(), CString::new("0").unwrap()],
                ).expect("failed execvp sleep");
            },
        );
    }
}
