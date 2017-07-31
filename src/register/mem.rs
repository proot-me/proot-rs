use std::path::{Path, PathBuf};
use errors::Result;
use register::Registers;

pub trait PtraceMemoryAllocator {
    fn alloc_mem() -> Result<()>;
}
