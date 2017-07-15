#[macro_use]
mod regs;

use libc::{pid_t, c_ulong, user_regs_struct};
use errors::Result;

pub type Word = c_ulong;

pub struct Registers {
    pub sys_num: usize,
    pub sys_arg_1: Word,
    pub sys_arg_2: Word,
    pub sys_arg_3: Word,
    pub sys_arg_4: Word,
    pub sys_arg_5: Word,
    pub sys_arg_6: Word,
    pub sys_result: i32,
}

impl Registers {
    pub fn retrieve(pid: pid_t) -> Result<Self> {
        Ok(Registers::from(&regs::fetch_all_regs(pid)?))
    }

    fn from(raw_regs: &user_regs_struct) -> Self {
        Self {
            sys_num: get_reg!(raw_regs, SysArgNum) as usize,
            sys_arg_1: get_reg!(raw_regs, SysArg1),
            sys_arg_2: get_reg!(raw_regs, SysArg2),
            sys_arg_3: get_reg!(raw_regs, SysArg3),
            sys_arg_4: get_reg!(raw_regs, SysArg4),
            sys_arg_5: get_reg!(raw_regs, SysArg5),
            sys_arg_6: get_reg!(raw_regs, SysArg6),
            sys_result: get_reg!(raw_regs, SysArgResult) as i32,
        }
    }
}
