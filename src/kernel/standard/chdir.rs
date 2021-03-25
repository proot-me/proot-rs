use crate::errors::Result;
use crate::kernel::exit::SyscallExitResult;

pub fn enter() -> Result<()> {
    Ok(())

    //    struct stat statl;
    //    char *tmp;
    //
    //    /* The ending "." ensures an error will be reported if
    //     * path does not exist or if it is not a directory.  */
    //    if (syscall_number == PR_chdir) {
    //        status = get_sysarg_path(tracee, path, SYSARG_1);
    //        if (status < 0)
    //            return SyscallExitResult::Value(status);
    //
    //        status = join_paths(2, oldpath, path, ".");
    //        if (status < 0)
    //            return SyscallExitResult::Value(status);
    //
    //        dirfd = AT_FDCWD;
    //    }
    //    else {
    //        strcpy(oldpath, ".");
    //        dirfd = peek_reg(tracee, CURRENT, SYSARG_1);
    //    }
    //
    //    status = translate_path(tracee, path, dirfd, oldpath, true);
    //    if (status < 0)
    //        return SyscallExitResult::Value(status);
    //
    //    status = lstat(path, &statl);
    //    if (status < 0)
    //        return SyscallExitResult::Value(status);
    //
    //    /* Check this directory is accessible.  */
    //    if ((statl.st_mode & S_IXUSR) == 0)
    //        return -EACCES;
    //
    //    /* Sadly this method doesn't detranslate statefully,
    //     * this means that there's an ambiguity when several
    //     * bindings are from the same host path:
    //     *
    //     *    $ proot -m /tmp:/a -m /tmp:/b fchdir_getcwd /a
    //     *    /b
    //     *
    //     *    $ proot -m /tmp:/b -m /tmp:/a fchdir_getcwd /a
    //     *    /a
    //     *
    //     * A solution would be to follow each file descriptor
    //     * just like it is done for cwd.
    //     */
    //
    //    status = detranslate_path(tracee, path, NULL);
    //    if (status < 0)
    //        return SyscallExitResult::Value(status);
    //
    //    /* Remove the trailing "/" or "/.".  */
    //    chop_finality(path);
    //
    //    tmp = talloc_strdup(tracee->fs, path);
    //    if (tmp == NULL) {
    //        return SyscallExitResult::Value(-ENOMEM);
    //    }
    //    TALLOC_FREE(tracee->fs->cwd);
    //
    //    tracee->fs->cwd = tmp;
    //    talloc_set_name_const(tracee->fs->cwd, "$cwd");
    //
    //    set_sysnum(tracee, PR_void);
    //    SyscallExitResult::Value(0)
}

pub fn exit() -> SyscallExitResult {
    // This syscall is fully emulated, see method `enter()` above.
    SyscallExitResult::None
}
