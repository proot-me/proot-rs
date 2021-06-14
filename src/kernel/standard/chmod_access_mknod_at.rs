use std::os::unix::prelude::RawFd;

use crate::errors::*;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysArg2};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let dirfd = tracee.regs.get(Current, SysArg(SysArg1)) as RawFd;
    let raw_path = tracee.regs.get_sysarg_path(SysArg2)?;

    let host_path = tracee.translate_path_at(dirfd, raw_path, true)?;

    tracee.regs.set_sysarg_path(
        SysArg2,
        &host_path,
        "during enter open translation, setting host path",
    )?;

    Ok(())
}
