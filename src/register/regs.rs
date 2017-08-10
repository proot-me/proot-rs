use std::ptr::null_mut;
use std::mem;
use std::fmt;
use libc::{c_void, user_regs_struct};
use errors::Result;
use nix::unistd::Pid;
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::{PTRACE_GETREGS, PTRACE_SETREGS};
use register::Word;

const VOID: usize = 0;

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum SysArgIndex {
    SysArg1 = 0,
    SysArg2,
    SysArg3,
    SysArg4,
    SysArg5,
    SysArg6,
}
use self::SysArgIndex::*;

#[derive(Debug, Copy, Clone)]
pub enum Register {
    SysNum,
    SysArg(SysArgIndex),
    SysResult,
    StackPointer,
}
use self::Register::*;


pub struct Registers {
    /// Pid of the tracee that it was generated from
    pid: Pid,
    /// Original general purpose registers; they must not be modified, except in `push_regs`
    raw_regs: user_regs_struct,
    /// Whether or not to only push `sys_arg_result` in `push_regs`.
    push_only_result: bool,
    sys_num: usize,
    sys_args: [Word; 6],
    sys_result: Word,
    stack_pointer: Word,
}

#[allow(dead_code)]
impl Registers {
    /// Extracts the most interesting registers from the raw structures,
    /// while keeping it for later purposes (see `push_reg`).
    pub fn from(pid: Pid, raw_regs: user_regs_struct) -> Self {
        Self {
            pid: pid,
            raw_regs: raw_regs,
            push_only_result: false,
            sys_num: get_reg!(raw_regs, SysNum) as usize,
            sys_args: [
                get_reg!(raw_regs, SysArg1),
                get_reg!(raw_regs, SysArg2),
                get_reg!(raw_regs, SysArg3),
                get_reg!(raw_regs, SysArg4),
                get_reg!(raw_regs, SysArg5),
                get_reg!(raw_regs, SysArg6),
            ],
            sys_result: get_reg!(raw_regs, SysResult),
            stack_pointer: get_reg!(raw_regs, StackPointer),
        }
    }

    /// Retrieves all tracee's general purpose registers.
    pub fn fetch_regs(pid: Pid) -> Result<Self> {
        let mut regs: user_regs_struct = unsafe { mem::zeroed() };
        let p_regs: *mut c_void = &mut regs as *mut _ as *mut c_void;

        // Notice the ? at the end, which is the equivalent of `try!`.
        // It will return the error if there is one.
        ptrace(PTRACE_GETREGS, pid, null_mut(), p_regs)?;

        Ok(Registers::from(pid, regs))
    }

    /// Pushes the cached general purpose registers back to the process,
    /// if necessary.
    pub fn push_regs(&mut self) -> Result<()> {
        if !self.where_changed() {
            return Ok(());
        }

        let mut modified_regs: user_regs_struct = self.raw_regs.clone();

        self.apply_to_raw_regs(&mut modified_regs, self.push_only_result);

        let p_regs: *mut c_void = &mut modified_regs as *mut _ as *mut c_void;

        ptrace(PTRACE_SETREGS, self.pid, null_mut(), p_regs)?;
        Ok(())
    }

    /// Applies the current values to `regs`.
    fn apply_to_raw_regs(&self, regs: &mut user_regs_struct, only_result: bool) {
        get_reg!(regs, SysResult) = self.sys_result as Word;

        // At the very end of a syscall, with regard to the entry,
        // only the result register can be modified by PRoot.
        if !only_result {
            get_reg!(regs, SysNum) = self.sys_num as Word;
            get_reg!(regs, SysArg1) = self.sys_args[0];
            get_reg!(regs, SysArg2) = self.sys_args[1];
            get_reg!(regs, SysArg3) = self.sys_args[2];
            get_reg!(regs, SysArg4) = self.sys_args[3];
            get_reg!(regs, SysArg5) = self.sys_args[4];
            get_reg!(regs, SysArg6) = self.sys_args[5];
            get_reg!(regs, StackPointer) = self.stack_pointer as Word;
        }
    }

    /// Checks whether at least one of the modifiable values is different from the original ones.
    /// If not, there is not point in pushing the registers.
    pub fn where_changed(&self) -> bool {
        return get_reg!(self.raw_regs, SysNum) != self.sys_num as u64 ||
            get_reg!(self.raw_regs, SysArg1) != self.sys_args[0] ||
            get_reg!(self.raw_regs, SysArg2) != self.sys_args[1] ||
            get_reg!(self.raw_regs, SysArg3) != self.sys_args[2] ||
            get_reg!(self.raw_regs, SysArg4) != self.sys_args[3] ||
            get_reg!(self.raw_regs, SysArg5) != self.sys_args[4] ||
            get_reg!(self.raw_regs, SysArg6) != self.sys_args[5] ||
            get_reg!(self.raw_regs, SysResult) != self.sys_result as u64 ||
            get_reg!(self.raw_regs, StackPointer) != self.stack_pointer;
    }

    #[inline]
    pub fn get(&self, register: Register) -> Word {
        match register {
            SysNum => self.sys_num as Word,
            SysArg(index) => self.sys_args[index as usize],
            SysResult => self.sys_result,
            StackPointer => self.stack_pointer,
        }
    }

    #[inline]
    pub fn get_raw(&self, register: Register) -> Word {
        match register {
            SysNum => get_reg!(self.raw_regs, SysNum),
            SysArg(SysArg1) => get_reg!(self.raw_regs, SysArg1),
            SysArg(SysArg2) => get_reg!(self.raw_regs, SysArg2),
            SysArg(SysArg3) => get_reg!(self.raw_regs, SysArg3),
            SysArg(SysArg4) => get_reg!(self.raw_regs, SysArg4),
            SysArg(SysArg5) => get_reg!(self.raw_regs, SysArg5),
            SysArg(SysArg6) => get_reg!(self.raw_regs, SysArg6),
            SysResult => get_reg!(self.raw_regs, SysResult),
            StackPointer => get_reg!(self.raw_regs, StackPointer),
        }
    }

    #[inline]
    pub fn set(&mut self, register: Register, new_value: Word) {
        match register {
            SysNum => self.sys_num = new_value as usize,
            SysArg(index) => self.sys_args[index as usize] = new_value,
            SysResult => self.sys_result = new_value,
            StackPointer => self.stack_pointer = new_value,
        };
    }

    #[inline]
    pub fn get_pid(&self) -> Pid {
        self.pid.clone()
    }

    #[inline]
    pub fn restore_stack_pointer(&mut self, enter_regs: Option<&mut Registers>) {
        match enter_regs {
            // At the exit stage, the original stack pointer is retrieved from the enter stage regs.
            Some(regs) => self.stack_pointer = regs.get_raw(StackPointer),
            // At the enter stage, we can use the raw regs directly.
            None => self.stack_pointer = self.get_raw(StackPointer),
        }
    }

    #[inline]
    pub fn push_only_result(&mut self, only_result: bool) {
        self.push_only_result = only_result
    }

    #[inline]
    pub fn void_syscall(&mut self) {
        self.sys_num = 0;
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(pid {}: syscall {} - args {:?}, result {}, stack-ptr {})",
            self.pid,
            self.sys_num,
            self.sys_args,
            self.sys_result,
            self.stack_pointer
        )
    }
}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(pid {}: syscall {} - args {:?}, result {}, stack-ptr {})",
            self.pid,
            self.sys_num,
            self.sys_args,
            self.sys_result,
            self.stack_pointer
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use nix::unistd::{Pid, execvp};
    use syscall::nr::NANOSLEEP;
    use utils::tests::fork_test;

    #[test]
    fn test_regs_where_changed() {
        let raw_regs: user_regs_struct = unsafe { mem::zeroed() };

        let regs = Registers::from(Pid::from_raw(-1), raw_regs);
        assert_eq!(false, regs.where_changed()); // no changes

        let mut regs = Registers::from(Pid::from_raw(-1), raw_regs);
        regs.sys_num = 123456;
        assert_eq!(true, regs.where_changed()); // syscall number change

        let mut regs = Registers::from(Pid::from_raw(-1), raw_regs);
        regs.sys_result = 123456;
        assert_eq!(true, regs.where_changed()); // sys arg result change

        let mut regs = Registers::from(Pid::from_raw(-1), raw_regs);
        regs.stack_pointer = 123456;
        assert_eq!(true, regs.where_changed()); // stack pointer

        for i in 0..6 {
            let mut regs = Registers::from(Pid::from_raw(-1), raw_regs);
            regs.sys_args[i] = 123456;
            assert_eq!(true, regs.where_changed()); // stack pointer
        }
    }


    #[test]
    fn test_fetch_regs_should_fail_test() {
        let regs = Registers::fetch_regs(Pid::from_raw(-1));
        assert!(regs.is_err());
    }

    #[test]
    fn test_fetch_regs_test() {
        fork_test(
            "/",
            // expecting a normal execution
            0,
            // parent
            |_, _, _| {
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
    fn test_fetch_regs_sysnum_sleep_test() {
        fork_test(
            "/",
            // expecting a normal execution
            0,
            // parent
            |regs, _, _| {
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

    #[test]
    /// Tests that `push_regs` works by voiding the NANOSLEEP syscall.
    /// It fails if the syscall is not cancelled (and in this case it will wait for 9999 secs),
    /// or if the tracee returns abruptly.
    fn test_push_regs_void_sysnum_sleep_test() {
        let mut sleep_exit = false;

        fork_test(
            "/",
            // expecting a normal execution
            0,
            // parent
            |mut regs, _, _| {
                if regs.sys_num == NANOSLEEP {
                    // we cancel the sleep call by voiding it
                    regs.void_syscall();
                    regs.push_regs().expect("pushing regs");
                    // the new syscall will be nanosleep's exit (with a sys num equal to 0)
                    sleep_exit = true;
                } else if sleep_exit {
                    // we restore the syscall number
                    regs.sys_num = NANOSLEEP;
                    // On successfully sleeping for the requested interval,
                    // nanosleep() returns 0.
                    regs.sys_result = 0;
                    regs.push_regs().expect("pushing regs");
                    return true;
                }

                return false;
            },
            // child
            || {
                // calling the sleep function, which should call the NANOSLEEP syscall
                execvp(
                    &CString::new("sleep").unwrap(),
                    &[CString::new(".").unwrap(), CString::new("9999").unwrap()],
                ).expect("failed execvp sleep");
            },
        );
    }
}
