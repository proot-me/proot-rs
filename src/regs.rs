use std::ptr::null_mut;
use std::mem;
use libc::{pid_t, c_void, user_regs_struct};
use nix::Result;
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::PTRACE_GETREGS;

pub enum Reg {
    SysArgNum,
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
    UserArg1
}

/// Specify the ABI registers (syscall argument passing, stack pointer).
/// See sysdeps/unix/sysv/linux/${ARCH}/syscall.S from the GNU C Library.
#[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
pub mod regs_offset {
    use super::*;

    pub fn get_reg(regs: &user_regs_struct, reg: Reg) -> u64 {
        match reg {
            Reg::SysArgNum      => regs.orig_rax,
            Reg::SysArg1        => regs.rdi,
            Reg::SysArg2        => regs.rsi,
            Reg::SysArg3        => regs.rdx,
            Reg::SysArg4        => regs.r10,
            Reg::SysArg5        => regs.r8,
            Reg::SysArg6        => regs.r9,
            Reg::SysArgResult   => regs.rax,
            Reg::StackPointer   => regs.rsp,
            Reg::InstrPointer   => regs.rip,
            Reg::RtldFini       => regs.rdx,
            Reg::StateFlags     => regs.eflags,
            Reg::UserArg1       => regs.rdi,
        }
        //todo: variant in case tracee->_regs[version].cs == 0x23
    }
}


#[cfg(all(target_os = "linux", any(target_arch = "x86")))]
pub mod regs_offset {
    use super::*;

    pub fn get_reg(regs: &user_regs_struct, reg: Reg) -> u64 {
        match reg {
            Reg::SysArgNum      => regs.orig_eax,
            Reg::SysArg1        => regs.ebx,
            Reg::SysArg2        => regs.ecx,
            Reg::SysArg3        => regs.edx,
            Reg::SysArg4        => regs.esi,
            Reg::SysArg5        => regs.edi,
            Reg::SysArg6        => regs.ebp,
            Reg::SysArgResult   => regs.eax,
            Reg::StackPointer   => regs.esp,
            Reg::InstrPointer   => regs.eip,
            Reg::RtldFini   => regs.edx,
            Reg::StateFlags     => regs.eflags,
            Reg::UserArg1       => regs.eax,
        }
    }
}

use self::regs_offset::get_reg;

/// Copy all @tracee's general purpose registers into a dedicated cache.
/// Returns either `Ok(regs)` or `Err(Sys(errno))` or `Err(InvalidPath)`.
pub fn fetch_regs(pid: pid_t) -> Result<user_regs_struct> {
    let mut regs: user_regs_struct = unsafe {mem::zeroed()};
    let p_regs: *mut c_void = &mut regs as *mut _ as *mut c_void;

    // Notice the ? at the end, which is the equivalent of `try!`.
    // It will return the error if there is one.
    ptrace(PTRACE_GETREGS, pid, null_mut(), p_regs) ?;

    Ok(regs)
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
    /// Tests that `fetch_regs` works on a simple syscall;
    /// the test is a success if the NANOSLEEP syscall is detected (with its corresponding signum).
    fn fetch_regs_sysnum_sleep_test() {
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
                                let sysnum = get_reg(&regs, Reg::SysArgNum);

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