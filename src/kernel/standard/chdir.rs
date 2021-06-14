use std::path::PathBuf;

use crate::errors::*;
use crate::filesystem::binding::Side;
use crate::process::tracee::Tracee;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysResult};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let sys_num = tracee.regs.get_sys_num(Current);
    let absolute_guest_path = if sys_num == sc::nr::CHDIR {
        let path = tracee.regs.get_sysarg_path(SysArg1)?;
        if path.is_relative() {
            let mut guest_path = PathBuf::from(tracee.fs.borrow().get_cwd());
            guest_path.push(path);
            guest_path
        } else {
            path
        }
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

    tracee.fs.borrow_mut().set_cwd(absolute_guest_path)?;

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
