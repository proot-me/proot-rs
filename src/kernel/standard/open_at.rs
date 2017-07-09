use nix::Result;

pub fn enter() -> Result<()> {
    Ok(())

    //                dirfd = peek_reg(tracee, CURRENT, SYSARG_1);
    //                flags = peek_reg(tracee, CURRENT, SYSARG_3);
    //
    //                status = get_sysarg_path(tracee, path, SYSARG_2);
    //                if (status < 0)
    //                break;
    //
    //                if (   ((flags & O_NOFOLLOW) != 0)
    //                || ((flags & O_EXCL) != 0 && (flags & O_CREAT) != 0))
    //                status = translate_path2(tracee, dirfd, path, SYSARG_2, SYMLINK);
    //                else
    //                status = translate_path2(tracee, dirfd, path, SYSARG_2, REGULAR);
}
