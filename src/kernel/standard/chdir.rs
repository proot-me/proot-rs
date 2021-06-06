use nix::unistd::AccessFlags;

use crate::errors::*;
use crate::filesystem::binding::Side;
use crate::filesystem::Canonicalizer;
use crate::filesystem::Translator;
use crate::process::tracee::Tracee;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysResult};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let sys_num = tracee.regs.get_sys_num(Current);
    let mut guest_path = if sys_num == sc::nr::CHDIR {
        tracee.regs.get_sysarg_path(SysArg1)?
    } else if sys_num == sc::nr::FCHDIR {
        tracee.get_path_from_fd(
            tracee.regs.get(Current, SysArg(SysArg1)) as i32,
            Side::Guest,
        )?
    } else {
        Err(Error::errno_with_msg(
            Errno::ENOSYS,
            format!(
                "sysno should be CHDIR({}) or FCHDIR({}), but got {}",
                sc::nr::CHDIR,
                sc::nr::FCHDIR,
                sys_num
            ),
        ))?
    };

    // The ending "." ensures an error will be reported if path does not exist or if
    // it is not a directory.
    guest_path.push(".");

    let host_path = tracee.fs.translate_path(&guest_path, true)?;

    // To change cwd to a dir, the tracee must have execute (`x`) permission to this
    // dir, FIXME: This may be wrong, because we need to check if tracee has
    // permission
    nix::unistd::access(&host_path, AccessFlags::X_OK)?;

    // TODO: this can be optimized
    let guest_path_canonical = tracee.fs.canonicalize(&guest_path, true)?;

    tracee.set_cwd(guest_path_canonical);

    // Avoid this syscall
    tracee
        .regs
        .cancel_syscall("Cancel chdir since it is fully emulated");

    Ok(())
}

pub fn exit(tracee: &mut Tracee) -> Result<()> {
    // This syscall is fully emulated, see method `enter()` above.

    tracee
        .regs
        .set(SysResult, 0u64, "update return value in chdir::exit()");
    tracee.regs.set_restore_original_regs(false);
    Ok(())
}
