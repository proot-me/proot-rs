use std::os::unix::prelude::RawFd;

use nix::fcntl::OFlag;

use crate::errors::*;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysArg2, SysArg3};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let dirfd = tracee.regs.get(Current, SysArg(SysArg1)) as RawFd;
    let raw_path = tracee.regs.get_sysarg_path(SysArg2)?;
    let flags = OFlag::from_bits_truncate(tracee.regs.get(Current, SysArg(SysArg3)) as _);

    debug!("openat(0x{:x?}, {:?}, {:?})", dirfd, raw_path, flags);

    let deref_final = !(flags.contains(OFlag::O_NOFOLLOW)
        || (flags.contains(OFlag::O_EXCL) && flags.contains(OFlag::O_CREAT)));

    let host_path = tracee.translate_path_at(dirfd, raw_path, deref_final)?;

    tracee.regs.set_sysarg_path(
        SysArg2,
        &host_path,
        "during enter open translation, setting host path",
    )?;
    // We don't need to modify SysArg1 because the SysArg2 is an absolute path now

    Ok(())
}
