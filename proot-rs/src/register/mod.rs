#[macro_use]
mod abi;
mod mem;
mod reader;
mod regs;
mod writer;

use libc::c_ulong;

pub type Word = c_ulong;

pub use self::mem::PtraceMemoryAllocator;
pub use self::reader::PtraceReader;
pub use self::regs::RegVersion::{self, *};
pub use self::regs::Register::*;
pub use self::regs::Registers;
pub use self::regs::SysArgIndex;
pub use self::regs::SysArgIndex::*;
pub use self::writer::PtraceWriter;
