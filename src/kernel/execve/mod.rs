#[macro_use]
mod macros;
mod elf;
mod load_info;
pub mod enter;
pub mod exit;
mod shebang;
mod loader;

use errors::Result;
use kernel::exit::SyscallExitResult;
use process::tracee::Tracee;
use kernel::execve::loader::LoaderFile;

pub fn enter(tracee: &mut Tracee, loader: &LoaderFile) -> Result<()> {
    enter::translate(tracee, loader)
}

pub fn exit(tracee: &mut Tracee) -> SyscallExitResult {
    exit::translate(tracee)
}
