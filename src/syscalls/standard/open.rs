use nix::Result;

pub fn enter() -> Result<()> {
    Ok(())

    //                 flags = peek_reg(tracee, CURRENT, SYSARG_2);
    //
    //                 if (   ((flags & O_NOFOLLOW) != 0)
    //                     || ((flags & O_EXCL) != 0 && (flags & O_CREAT) != 0))
    //                 status = translate_sysarg(tracee, SYSARG_1, SYMLINK);
    //                 else
    //                 status = translate_sysarg(tracee, SYSARG_1, REGULAR);
}