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
use process::tracee::Tracee;
use kernel::execve::loader::LoaderFile;

pub fn enter(tracee: &mut Tracee, regs: &mut Registers, loader: &LoaderFile) -> Result<()> {
    enter::translate(tracee, regs, loader)
}

pub fn exit(tracee: &Tracee, regs: &Registers) -> SyscallExitResult {
    exit::translate(tracee, regs)
}
