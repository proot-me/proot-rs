use std::ptr::null_mut;
use libc::{pid_t, c_void};
use nix::Result;
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::PTRACE_GETREGS;

/// Helper that transforms a Rust structure into a C structure
/// by adding `#[repr(C)]` on top of it, and making it copyable and cloneable.
/// The unroll part is there to gain (code) space by grouping fields
/// that have the same type. For instance:
/// pub [a, b, c] : u64
/// will be translated into:
/// pub a : u64,
/// pub b : u64,
/// pub c : u64.
/// It also implements the new() method (though it only works if none of the fields are arrays).
macro_rules! unroll_and_structure {
    ($($(#[$attr:meta])*
        pub struct $i:ident {
            $(pub [ $( $field:ident ),* ]: $tt:ty),*
        }
    )*) => ($(
        #[repr(C)]
        $(#[$attr])*
        pub struct $i { $( $(pub $field: $tt),*),* }
        impl $i {
            pub fn new() -> $i {
                $i {
                    $( $( $field: 0),*),*
                }
            }
        }
        impl Copy for $i {}
        impl Clone for $i {
            fn clone(&self) -> $i { *self }
        }
    )*)
}

/// The following structures are there to get the offset of the register's fields
/// (syscall number, arg1, arg2, etc...) in the current architecture's structures.
///
/// Reminder: the order in which the fields are declared is paramount.
/// `[repr(C)]` ensures that it stays the same when transformed in a C struct.

#[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
pub mod regs_structs {
    unroll_and_structure! {
        #[derive(Debug)]
        pub struct user_regs_struct {
            pub [r15, r14, r13, r12, rbp, rbx, r11, r10, r9, r8, rax, rcx, rdx, rsi, rdi, orig_rax,
            rip, cs, eflags, rsp, ss, fs_base, gs_base, ds, es, fs, gs]: u64
        }
    }
}

#[cfg(all(target_os = "linux", any(target_arch = "x86")))]
mod regs_structs {
}

#[cfg(all(target_os = "linux", any(target_arch = "arm")))]
mod regs_structs {
}

use self::regs_structs::user_regs_struct;

/*
pub enum Reg {
    SysArgNum = 0,
    SysArg1,
    SysArg2,
    SysArg3,
    SysArg4,
    SysArg5,
    SysArg6,
    SysArgResult,
    StackPointer,
    InstrPointer,
    RtldFini,
    StateFlags,
    UserArg1,
}
*/

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
    //TODO: x86 ABI correspondence
}

#[cfg(all(target_os = "linux", any(target_arch = "arm")))]
#[macro_use]
pub mod regs_offset {
    //TODO: arm ABI correspondence
}

/// Copy all @tracee's general purpose registers into a dedicated cache.
/// Returns either `Ok(regs)` or `Err(Sys(errno))` or `Err(InvalidPath)`.
pub fn fetch_regs(pid: pid_t) -> Result<user_regs_struct> {
    let mut regs: user_regs_struct = user_regs_struct::new();
    let p_regs: *mut c_void = &mut regs as *mut _ as *mut c_void;

    let ret = ptrace(PTRACE_GETREGS, pid, null_mut(), p_regs);

    // The Ok(_) signal (Ok(0) usually) is mapped to the registers,
    // otherwise the error is directly returned.
    ret.map(|_| regs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::null_mut;
    use std::ffi::CString;
    use nix::unistd::{getpid, fork, execvp, ForkResult};
    use nix::sys::signal::{kill};
    use nix::sys::signal::Signal::{SIGSTOP};
    use nix::sys::ptrace::ptrace;
    use nix::sys::ptrace::ptrace::PTRACE_TRACEME;
    use nix::sys::wait::{waitpid, __WALL};
    use nix::sys::wait::WaitStatus::*;
    use nix::sys::ptrace::ptrace::PTRACE_SYSCALL;
    use proot::InfoBag;
    use tracee::Tracee;
    use syscall::nr::NANOSLEEP;

    #[test]
    fn fetch_regs_test() {
        match fork().expect("fork in test") {
            ForkResult::Parent { child } => {
                // the parent will wait for the child's signal before calling set_ptrace_options
                assert_eq!(waitpid(-1, Some(__WALL)).expect("event loop waitpid"), Stopped(child, SIGSTOP));

                let ret = fetch_regs(child);
                assert!(ret.is_ok());

                restart(child);
                end(child);
            }
            ForkResult::Child => {
                ptrace(PTRACE_TRACEME, 0, null_mut(), null_mut()).expect("test ptrace traceme");
                // we use a SIGSTOP to synchronise both processes
                kill(getpid(), SIGSTOP).expect("test child sigstop");
            }
        }
    }
    #[test]
    fn fetch_regs_should_fail_test() {
        let ret = fetch_regs(-1);
        assert!(ret.is_err());
    }

    #[test]
    fn get_reg_sysnum_sleep_test() {
        match fork().expect("fork in test") {
            ForkResult::Parent { child } => {
                let info_bag = &mut InfoBag::new();
                let tracee = Tracee::new(child);

                // the parent will wait for the child's signal before calling set_ptrace_options
                assert_eq!(waitpid(-1, Some(__WALL)).expect("event loop waitpid"), Stopped(child, SIGSTOP));
                tracee.set_ptrace_options(info_bag);

                restart(child);

                // we loop until the NANOSLEEP syscall is called
                loop {
                    match waitpid(-1, Some(__WALL)).expect("event loop waitpid") {
                        PtraceSyscall(pid) => {
                            assert_eq!(pid, child);
                            let maybe_regs = fetch_regs(child);
                            assert!(maybe_regs.is_ok());

                            if maybe_regs.is_ok() {
                                let regs = maybe_regs.unwrap();
                                let sysnum = get_reg!(regs, SysArgNum);

                                if sysnum == NANOSLEEP as u64 {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        Exited(_, _) => { assert!(false) }
                        Signaled(_, _, _) => { assert!(false) }
                        _ => {}
                    }
                    restart(child);
                }

                restart(child);
                end(child);
            }
            ForkResult::Child => {
                ptrace(PTRACE_TRACEME, 0, null_mut(), null_mut()).expect("test ptrace traceme");
                // we use a SIGSTOP to synchronise both processes
                kill(getpid(), SIGSTOP).expect("test child sigstop");

                // calling the sleep function,
                // which should call the NANOSLEEP syscall
                execvp(&CString::new("sleep").unwrap(), &[CString::new(".").unwrap(), CString::new("0").unwrap()])
                    .expect("failed execvp sleep");
            }
        }
    }

    /// Restarts a child process
    fn restart(child: pid_t) {
        ptrace(PTRACE_SYSCALL, child, null_mut(), null_mut()).expect("exit tracee with exit stage");
    }

    /// Waits/restarts a child process until it stops.
    fn end(child: pid_t) {
        loop {
            match waitpid(-1, Some(__WALL)).expect("waitpid") {
                Exited(pid, exit_status) => {
                    assert_eq!(pid, child);

                    // the tracee should have exited with an OK status (exit code 0)
                    assert_eq!(exit_status, 0);
                    break;
                }
                _ => {
                    // restarting the tracee
                    ptrace(PTRACE_SYSCALL, child, null_mut(), null_mut()).expect("exit tracee with exit stage");
                }
            }
        }
    }
}