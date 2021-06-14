use crate::errors::*;

use crate::filesystem::Translator;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{PtraceReader, SysArg1};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let raw_path = tracee.regs.get_sysarg_path(SysArg1)?;

    debug!("dir_link_attr({:?})", raw_path);
    let host_path = tracee.fs.borrow().translate_path(raw_path, false)?;

    tracee.regs.set_sysarg_path(
        SysArg1,
        &host_path,
        "during enter open translation, setting host path",
    )?;
    Ok(())
}
