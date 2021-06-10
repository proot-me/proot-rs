use crate::errors::Result;
use crate::register::Word;
use libc::user_regs_struct;
use nix::sys::ptrace;
use nix::unistd::Pid;
use std::fmt;

const VOID: Word = Word::MAX;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RegVersion {
    Current = 0,  // indicates current registers value
    Original = 1, // the original registers value of the syscall
    Modified = 2, // registers value modified during syscall enter translation
}
use self::RegVersion::*;

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

#[derive(Debug)]
pub struct Registers {
    /// Pid of the tracee that it was generated from
    pid: Pid,
    registers: [Option<user_regs_struct>; 3],
    regs_were_changed: bool,
    restore_original_regs: bool,
}

#[allow(dead_code)]
impl Registers {
    /// Creates an empty registers bundle.
    pub fn new(pid: Pid) -> Self {
        Self {
            pid: pid,
            registers: [None, None, None],
            regs_were_changed: false,
            restore_original_regs: false,
        }
    }

    #[cfg(test)]
    /// Same, but with the initial regs. Useful for tests.
    pub fn from(pid: Pid, raw_regs: user_regs_struct) -> Self {
        Self {
            pid: pid,
            registers: [Some(raw_regs), None, None],
            regs_were_changed: false,
            restore_original_regs: false,
        }
    }

    /// Retrieves a value from one of the registers.
    ///
    /// It does not require the registers to be mutable,
    /// so we allow any register version (even original).
    ///
    /// # Safety
    ///
    /// Be sure that the register version you're asking for exists,
    /// otherwise the program will simply panic.
    /// It is like this so that a backtrace can be retrieved,
    /// in order to remedy the issue so that it doesn't happen again.
    #[inline]
    pub fn get(&self, version: RegVersion, register: Register) -> Word {
        let raw_regs = self.get_regs(version);

        self.get_raw(raw_regs, register)
    }

    /// Modifies the value of one of the `Current` registers.
    ///
    /// If `new_value` is the same as the current one, `regs_were_changed`
    /// is not toggled, in order to avoid unnecessary `push_regs`.
    ///
    /// # Safety
    ///
    /// Be sure that the `Current` registers exist, otherwise the program will
    /// panic. It is like this so that a backtrace can be retrieved,
    /// in order to remedy the issue so that it doesn't happen again.
    #[inline]
    pub fn set(&mut self, register: Register, new_value: Word, justification: &'static str) {
        let current_value = self.get(Current, register);

        //TODO: log DEBUG
        debug!(
            "-- {}, Modifying current reg: {:?}, current_value: {:#x}, new_value: {:#x}, {}",
            self.pid, register, current_value, new_value, justification
        );

        if current_value == new_value {
            return;
        }
        self.set_raw(register, new_value);
        self.regs_were_changed = true;
    }

    /// Saves the `Current` registers into the given `version` ones.
    ///
    /// This is the only way to modify the `Original` and `Modified` registers
    /// in this structure.
    ///
    /// Requires the `Current` registers to be defined.
    #[inline]
    pub fn save_current_regs(&mut self, version: RegVersion) {
        if version != Current {
            let current_regs = *self.get_regs(Current);

            self.registers[version as usize] = Some(current_regs);
        }
    }

    /// Retrieves all tracee's general purpose registers, and stores them
    /// in the `Current` registers.
    pub fn fetch_regs(&mut self) -> Result<()> {
        // Notice the ? at the end, which is the equivalent of `try!`.
        // It will return the error if there is one.
        let regs: user_regs_struct = ptrace::getregs(self.pid)?;

        self.registers[Current as usize] = Some(regs);
        Ok(())
    }

    /// Pushes the `Current` cached general purpose registers back to
    /// the process, if necessary.
    ///
    /// Requires `Current` registers to be defined, and `Original` if
    /// `restore_original_regs` is enabled.
    pub fn push_regs(&mut self) -> Result<()> {
        if !self.regs_were_changed {
            return Ok(());
        }

        if self.restore_original_regs {
            self.restore_regs();
        }

        let pid = self.pid;
        let current_regs = self.get_mut_regs(Current);

        debug!("Push regs: {:x?}", current_regs);
        ptrace::setregs(pid, *current_regs)?;
        Ok(())
    }

    /// Utility function to retrieve the corresponding register's value
    /// from a `user_regs_struct` structure.
    ///
    /// This function relies on the ABI mapping implemented through the
    /// `get_reg!` macro.
    #[inline]
    fn get_raw(&self, raw_regs: &user_regs_struct, register: Register) -> Word {
        match register {
            SysNum => get_reg!(raw_regs, SysNum),
            SysArg(SysArg1) => get_reg!(raw_regs, SysArg1),
            SysArg(SysArg2) => get_reg!(raw_regs, SysArg2),
            SysArg(SysArg3) => get_reg!(raw_regs, SysArg3),
            SysArg(SysArg4) => get_reg!(raw_regs, SysArg4),
            SysArg(SysArg5) => get_reg!(raw_regs, SysArg5),
            SysArg(SysArg6) => get_reg!(raw_regs, SysArg6),
            SysResult => get_reg!(raw_regs, SysResult),
            StackPointer => get_reg!(raw_regs, StackPointer),
        }
    }

    /// Utility function to modify the corresponding register's value
    /// of a `user_regs_struct` structure.
    ///
    /// Though only the `Current` regs are allowed to be modified directly
    /// (the others are created through saves), so this function only
    /// applies to the `Current` registers.
    ///
    /// This function relies on the ABI mapping implemented through the
    /// `get_reg!` macro.
    ///
    /// Requires the `Current` registers to be defined.
    #[inline]
    fn set_raw(&mut self, register: Register, new_value: Word) {
        let raw_regs = self.get_mut_regs(Current);

        match register {
            SysNum => get_reg!(raw_regs, SysNum) = new_value,
            SysArg(SysArg1) => get_reg!(raw_regs, SysArg1) = new_value,
            SysArg(SysArg2) => get_reg!(raw_regs, SysArg2) = new_value,
            SysArg(SysArg3) => get_reg!(raw_regs, SysArg3) = new_value,
            SysArg(SysArg4) => get_reg!(raw_regs, SysArg4) = new_value,
            SysArg(SysArg5) => get_reg!(raw_regs, SysArg5) = new_value,
            SysArg(SysArg6) => get_reg!(raw_regs, SysArg6) = new_value,
            SysResult => get_reg!(raw_regs, SysResult) = new_value,
            StackPointer => get_reg!(raw_regs, StackPointer) = new_value,
        };
    }

    /// Restore the current regs with the original ones.
    ///
    /// Requires both `Current` and `Original` regs to be defined.
    #[inline]
    fn restore_regs(&mut self) {
        let original_regs = self.registers[Original as usize].unwrap(); // get a copy of original regs
        let current_regs = self.registers[Current as usize].as_mut().unwrap();

        get_reg!(current_regs, SysNum) = get_reg!(original_regs, SysNum);
        get_reg!(current_regs, SysArg1) = get_reg!(original_regs, SysArg1);
        get_reg!(current_regs, SysArg2) = get_reg!(original_regs, SysArg2);
        get_reg!(current_regs, SysArg3) = get_reg!(original_regs, SysArg3);
        get_reg!(current_regs, SysArg4) = get_reg!(original_regs, SysArg4);
        get_reg!(current_regs, SysArg5) = get_reg!(original_regs, SysArg5);
        get_reg!(current_regs, SysArg6) = get_reg!(original_regs, SysArg6);
        get_reg!(current_regs, StackPointer) = get_reg!(original_regs, StackPointer);
    }

    #[inline]
    pub fn get_pid(&self) -> Pid {
        self.pid
    }

    #[inline]
    fn get_regs(&self, version: RegVersion) -> &user_regs_struct {
        match self.registers[version as usize] {
            Some(ref regs) => regs,
            None => unreachable!(),
        }
    }

    #[inline]
    fn get_mut_regs(&mut self, version: RegVersion) -> &mut user_regs_struct {
        match self.registers[version as usize] {
            Some(ref mut regs) => regs,
            None => unreachable!(),
        }
    }

    /// Little utility method to quickly retrieve the syscall number.
    #[inline]
    pub fn get_sys_num(&self, version: RegVersion) -> usize {
        self.get(version, SysNum) as usize
    }

    /// Little utility method to quickly modify the syscall number.
    #[inline]
    pub fn set_sys_num(&mut self, new_value: usize, justification: &'static str) {
        self.set(SysNum, new_value as Word, justification);
    }

    /// Little utility method to quickly void the syscall number.
    #[inline]
    pub fn cancel_syscall(&mut self, justification: &'static str) {
        self.set(SysNum, VOID, justification);
    }

    #[inline]
    pub fn set_restore_original_regs(&mut self, restore_original_regs: bool) {
        self.restore_original_regs = restore_original_regs;
    }

    /// Little utility method to quickly restore the original version
    /// of a register.
    ///
    /// Requires both `Original` and `Current` registers to be defined.
    #[inline]
    pub fn restore_original(&mut self, register: Register, justification: &'static str) {
        let original_value = self.get(Original, register);

        self.set(register, original_value, justification);
    }

    #[inline]
    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let current_regs = &self.registers[Current as usize].unwrap();

        write!(
            f,
            "(pid {}: syscall {} - args [{}, {}, {}, {}, {}, {}], result {}, stack-ptr {})",
            self.pid,
            get_reg!(current_regs, SysNum),
            get_reg!(current_regs, SysArg1),
            get_reg!(current_regs, SysArg2),
            get_reg!(current_regs, SysArg3),
            get_reg!(current_regs, SysArg4),
            get_reg!(current_regs, SysArg5),
            get_reg!(current_regs, SysArg6),
            get_reg!(current_regs, SysResult),
            get_reg!(current_regs, StackPointer),
        )
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.display(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::{fork_test, get_test_rootfs_path};
    use nix::unistd::{execvp, Pid};
    use sc::nr::{CLOCK_NANOSLEEP, NANOSLEEP};
    use std::{ffi::CString, mem};

    #[test]
    fn test_regs_where_changed() {
        let mut regs = Registers::from(Pid::from_raw(-1), unsafe { mem::zeroed() });

        assert_eq!(false, regs.regs_were_changed);

        regs.set(SysNum, 123456, "");

        assert_eq!(true, regs.regs_were_changed);
        assert_eq!(123456, regs.get(Current, SysNum));
    }

    #[test]
    fn test_fetch_regs_should_fail_test() {
        let mut regs = Registers::new(Pid::from_raw(-1));

        assert!(regs.fetch_regs().is_err());
    }

    #[test]
    fn test_fetch_regs_test() {
        let rootfs_path = get_test_rootfs_path();

        fork_test(
            rootfs_path,
            // expecting a normal execution
            0,
            // parent
            |_, _| {
                // we stop on the first syscall;
                // the fact that no panic was sparked until now means that the regs were OK
                true
            },
            // child
            || {
                // calling the sleep function, which should call the NANOSLEEP syscall
                execvp(
                    &CString::new("sleep").unwrap(),
                    &[CString::new(".").unwrap(), CString::new("0").unwrap()],
                )
                .expect("failed execvp sleep");
            },
        );
    }

    #[test]
    /// Tests that `fetch_regs` works on a simple syscall;
    /// the test is a success if the NANOSLEEP syscall is detected (with its
    /// corresponding signum).
    fn test_fetch_regs_sysnum_sleep_test() {
        let rootfs_path = get_test_rootfs_path();

        fork_test(
            rootfs_path,
            // expecting a normal execution
            0,
            // parent
            |tracee, _| {
                // we only stop when the NANOSLEEP syscall is detected
                tracee.regs.get_sys_num(Current) == NANOSLEEP
            },
            // child
            || {
                // calling the sleep function, which should call the NANOSLEEP syscall
                execvp(
                    &CString::new("sleep").unwrap(),
                    &[CString::new(".").unwrap(), CString::new("0").unwrap()],
                )
                .expect("failed execvp sleep");
            },
        );
    }

    #[test]
    /// Tests that `push_regs` works by voiding the NANOSLEEP syscall.
    /// It fails if the syscall is not cancelled (and in this case it will wait
    /// for 9999 secs), or if the tracee returns abruptly.
    fn test_push_regs_void_sysnum_sleep_test() {
        let rootfs_path = get_test_rootfs_path();
        let mut sleep_exit = false;

        fork_test(
            rootfs_path,
            // expecting a normal execution
            0,
            // parent
            |tracee, _| {
                let sys_num = tracee.regs.get_sys_num(Current);
                if sys_num == NANOSLEEP || sys_num == CLOCK_NANOSLEEP {
                    // NANOSLEEP enter stage
                    tracee.regs.set_restore_original_regs(false);
                    tracee.regs.save_current_regs(Original);

                    // we cancel the sleep call by voiding it
                    tracee
                        .regs
                        .cancel_syscall("cancel sleep for push regs test");
                    tracee.regs.push_regs().expect("pushing regs");

                    // the new syscall will be nanosleep's exit (with a sys num equal to 0)
                    sleep_exit = true;
                } else if sleep_exit {
                    // NANOSLEEP exit stage
                    tracee.regs.set_restore_original_regs(true);
                    tracee.regs.set(SysResult, 0, "simulate successful sleep");
                    tracee.regs.push_regs().expect("pushing regs");
                    return true;
                }

                false
            },
            // child
            || {
                // calling the sleep function, which should call the NANOSLEEP syscall
                execvp(
                    &CString::new("sleep").unwrap(),
                    &[CString::new(".").unwrap(), CString::new("9999").unwrap()],
                )
                .expect("failed execvp sleep");
            },
        );
    }
}
