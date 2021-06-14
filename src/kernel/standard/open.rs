use nix::fcntl::OFlag;

use crate::errors::*;

use crate::filesystem::Translator;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysArg2};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let raw_path = tracee.regs.get_sysarg_path(SysArg1)?;

    let flags = OFlag::from_bits_truncate(tracee.regs.get(Current, SysArg(SysArg2)) as _);

    debug!("open({:?}, {:?})", raw_path, flags);
    let deref_final = !(flags.contains(OFlag::O_NOFOLLOW)
        || (flags.contains(OFlag::O_EXCL) && flags.contains(OFlag::O_CREAT)));
    let host_path = tracee.fs.borrow().translate_path(raw_path, deref_final)?;

    tracee.regs.set_sysarg_path(
        SysArg1,
        &host_path,
        "during enter open translation, setting host path",
    )?;

    Ok(())
}
