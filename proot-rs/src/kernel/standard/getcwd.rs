use std::os::unix::prelude::OsStrExt;

use libc::c_void;

use crate::errors::*;

use crate::filesystem::Translator;
use crate::process::tracee::Tracee;
use crate::register::{Original, SysArg, SysArg1, SysArg2, SysResult};
use crate::register::{PtraceWriter, Word};

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

    let fs_r = tracee.fs.borrow();
    let guest_path = fs_r.get_cwd();
    // we need to ensure cwd still exists
    tracee
        .fs
        .borrow()
        .translate_absolute_path(&guest_path, true)?
        .1
        .metadata()?;

    let bytes = guest_path.as_os_str().as_bytes();
    let real_size = bytes.len() + 1;
    if real_size > size {
        return Err(Error::errno(Errno::ERANGE));
    }

    tracee
        .regs
        .write_data(buf_addr as *mut c_void, bytes, true)?;

    // Unlike the description in the getcwd() man page, in the Linux kernel, the
    // getcwd() system call returns the length of the buffer filled.
    // https://elixir.bootlin.com/linux/v5.10.43/source/fs/d_path.c#L412
    tracee.regs.set(
        SysResult,
        real_size as Word,
        "update return value in getcwd::exit()",
    );
    Ok(())
}
