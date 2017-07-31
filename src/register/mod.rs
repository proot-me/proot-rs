#[macro_use]
mod abi;
mod regs;
pub mod reader;
pub mod writer;
pub mod mem;

use libc::c_ulong;

pub type Word = c_ulong;

pub use self::regs::{Registers, SysArgIndex};
