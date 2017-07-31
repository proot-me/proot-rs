use std::ptr::null_mut;
use std::mem;
use libc::{c_void, user_regs_struct};
use errors::Result;
use nix::unistd::Pid;
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::PTRACE_GETREGS;
use register::Word;

#[derive(Debug, Copy, Clone)]
pub enum SysArgIndex {
    SysArg1 = 0,
    SysArg2,
    SysArg3,
    SysArg4,
    SysArg5,
    SysArg6,
}

pub struct Registers {
    pub pid: Pid,
    pub raw_regs: user_regs_struct,
    pub sys_num: usize,
    pub sys_args: [Word; 6],
    pub sys_arg_result: i32,
}

impl Registers {
    pub fn retrieve(pid: Pid) -> Result<Self> {
        Ok(Registers::from(pid, fetch_all_regs(pid)?))
    }

    pub fn from(pid: Pid, raw_regs: user_regs_struct) -> Self {
        Self {
            pid: pid,
            raw_regs: raw_regs,
            sys_num: get_reg!(raw_regs, SysArgNum) as usize,
            sys_args: [
                get_reg!(raw_regs, SysArg1),
                get_reg!(raw_regs, SysArg2),
                get_reg!(raw_regs, SysArg3),
                get_reg!(raw_regs, SysArg4),
                get_reg!(raw_regs, SysArg5),
                get_reg!(raw_regs, SysArg6),
            ],
            sys_arg_result: get_reg!(raw_regs, SysArgResult) as i32,
        }
    }

    pub fn get_arg(&self, index: SysArgIndex) -> Word {
        self.sys_args[index as usize]
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
