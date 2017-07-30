#[macro_use]
mod macros;
mod elf;
mod load_info;
pub mod enter;
pub mod exit;
mod shebang;
mod loader;

use errors::Result;
use register::Registers;
use kernel::exit::SyscallExitResult;
use filesystem::fs::FileSystem;
use process::tracee::Tracee;
use kernel::execve::loader::LoaderFile;

pub fn enter(
    fs: &FileSystem,
    tracee: &mut Tracee,
    regs: &Registers,
    loader: &LoaderFile,
) -> Result<()> {
    enter::translate(fs, tracee, regs, loader)
}

pub fn exit() -> SyscallExitResult {
    exit::translate()
}
