use std::os::unix::prelude::RawFd;

use crate::errors::*;
use crate::filesystem::ext::PathExt;
use crate::kernel::standard::link_rename;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysArg2, SysArg3, SysArg4};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let olddirfd = tracee.regs.get(Current, SysArg(SysArg1)) as RawFd;
    let newdirfd = tracee.regs.get(Current, SysArg(SysArg3)) as RawFd;
    let old_path = tracee.regs.get_sysarg_path(SysArg2)?;
    let new_path = tracee.regs.get_sysarg_path(SysArg4)?;

    let deref_final = old_path.with_trailing_slash();

    let old_host_path = tracee.translate_path_at(olddirfd, old_path, deref_final)?.1;
    let new_host_path = tracee.translate_path_at(newdirfd, new_path, false)?.1;

    tracee.regs.set_sysarg_path(
        SysArg2,
        &old_host_path,
        "during enter open translation, setting host path",
    )?;
    tracee.regs.set_sysarg_path(
        SysArg4,
        &new_host_path,
        "during enter open translation, setting host path",
    )?;

    Ok(())
}

pub fn exit(tracee: &mut Tracee) -> Result<()> {
    link_rename::exit(tracee)
}
