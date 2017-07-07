use nix::Result;

pub fn enter() -> Result<()> {
    Ok(())

    //                 flags = peek_reg(tracee, CURRENT, SYSARG_3);
    //
    //                 if ((flags & IN_DONT_FOLLOW) != 0)
    //                     status = translate_sysarg(tracee, SYSARG_2, SYMLINK);
    //                 else
    //                     status = translate_sysarg(tracee, SYSARG_2, REGULAR);
}
