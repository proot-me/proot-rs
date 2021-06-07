use std::os::unix::prelude::RawFd;

use nix::fcntl::OFlag;

use crate::errors::*;
use crate::filesystem::binding::Side;
use crate::filesystem::Substitutor;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysArg2, SysArg3};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let dirfd = tracee.regs.get(Current, SysArg(SysArg1)) as RawFd;
    let raw_path = tracee.regs.get_sysarg_path(SysArg2)?;
    let flags = OFlag::from_bits_truncate(tracee.regs.get(Current, SysArg(SysArg3)) as _);

    debug!("openat(0x{:x?}, {:?}, {:?})", dirfd, raw_path, flags);

    let guest_path = if raw_path.is_absolute() {
        raw_path
    } else {
        let mut dir_guest_path = tracee.get_path_from_fd(dirfd, Side::Guest)?;
        let dir_host_path = tracee.fs.borrow().translate_path(&dir_guest_path, true)?;
        if !dir_host_path
            .symlink_metadata()
            .errno(Errno::ENOTDIR)?
            .is_dir()
        {
            return Err(Error::errno_with_msg(
                Errno::ENOTDIR,
                format!("The path is not a dir: {:?}", dir_host_path),
            ));
        }
        dir_guest_path.push(raw_path);
        dir_guest_path
    };

    let host_path = if flags.contains(OFlag::O_NOFOLLOW)
        || (flags.contains(OFlag::O_EXCL) && flags.contains(OFlag::O_CREAT))
    {
        tracee.fs.borrow().translate_path(&guest_path, false)?
    } else {
        tracee.fs.borrow().translate_path(&guest_path, true)?
    };
    tracee.regs.set_sysarg_path(
        SysArg2,
        &host_path,
        "during enter open translation, setting host path",
    )?;
    // We don't need to modify SysArg1 because the SysArg2 is an absolute path now

    Ok(())
}
