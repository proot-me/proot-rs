#[macro_use]
mod abi;
mod regs;
mod reader;
mod writer;
mod mem;

use libc::c_ulong;

pub type Word = c_ulong;

pub use self::regs::{Registers, SysArgIndex};
pub use self::reader::PtraceReader;
pub use self::writer::PtraceWriter;
pub use self::mem::PtraceMemoryAllocator;
