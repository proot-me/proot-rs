use std::os::unix::prelude::OsStrExt;

use libc::c_void;

use crate::errors::*;
use crate::filesystem::Translator;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Original, SysArg, SysArg1, SysArg2, SysResult};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    tracee
        .regs
        .cancel_syscall("Cancel getcwd and return our own value");
    Ok(())
}

pub fn exit(tracee: &mut Tracee) -> Result<()> {
    let buf_addr = tracee.regs.get(Original, SysArg(SysArg1));
    let size = tracee.regs.get(Original, SysArg(SysArg2)) as usize;
    if buf_addr == 0 || size == 0 {
        return Err(Error::errno(Errno::EINVAL));
    }

    // ensure cwd still exists
    let guest_path = tracee.get_cwd();
    tracee.fs.translate_path(guest_path, true)?;

    let bytes = guest_path.as_os_str().as_bytes();
    if bytes.len() + 1 > size {
        return Err(Error::errno(Errno::ERANGE));
    }

    error!("{:?}", guest_path);

    tracee
        .regs
        .write_data(buf_addr as *mut c_void, bytes, true)?;

    tracee
        .regs
        .set(SysResult, 0u64, "update return value in getcwd::exit()");
    tracee.regs.set_restore_original_regs(false);

    Ok(())
}
