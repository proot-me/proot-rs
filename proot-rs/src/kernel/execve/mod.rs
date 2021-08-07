#[macro_use]
mod macros;
mod binfmt;
pub mod enter;
pub mod exit;
pub mod load_info;
pub mod loader;
mod params;

use crate::errors::Result;
use crate::kernel::execve::loader::LoaderFile;
use crate::process::tracee::Tracee;

pub fn enter(tracee: &mut Tracee, loader: &dyn LoaderFile) -> Result<()> {
    enter::translate(tracee, loader)
}

pub fn exit(tracee: &mut Tracee) -> Result<()> {
    exit::translate(tracee)
}
