#[macro_use]
mod regs;
pub mod reader;

use libc::{c_ulong, user_regs_struct};
use nix::unistd::Pid;
use errors::Result;

pub type Word = c_ulong;

#[derive(Debug, Copy, Clone)]
pub enum SysArgIndex {
    SysArg1 = 0,
    SysArg2 = 1,
    SysArg3 = 2,
    SysArg4 = 3,
    SysArg5 = 4,
    SysArg6 = 5
}

pub struct Registers {
    pid: Pid,
    sys_num: usize,
    sys_args: [Word; 6],
    sys_arg_result: i32,
}

impl Registers {
    pub fn retrieve(pid: Pid) -> Result<Self> {
        Ok(Registers::from(pid, &regs::fetch_all_regs(pid)?))
    }

    fn from(pid: Pid, raw_regs: &user_regs_struct) -> Self {
        Self {
            pid: pid,
            sys_num: get_reg!(raw_regs, SysArgNum) as usize,
            sys_args: [
                get_reg!(raw_regs, SysArg1),
                get_reg!(raw_regs, SysArg2),
                get_reg!(raw_regs, SysArg3),
                get_reg!(raw_regs, SysArg4),
                get_reg!(raw_regs, SysArg5),
                get_reg!(raw_regs, SysArg6)
            ],
            sys_arg_result: get_reg!(raw_regs, SysArgResult) as i32,
        }
    }

    #[inline]
    pub fn get_sys_num(&self) -> usize {
        self.sys_num
    }

    #[inline]
    pub fn get_sys_arg_result(&self) -> i32 {
        self.sys_arg_result
    }

    #[inline]
    fn get_arg(&self, index: SysArgIndex) -> Word {
        self.sys_args[index as usize]
    }
}
