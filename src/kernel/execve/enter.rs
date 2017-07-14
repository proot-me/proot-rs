use libc::pid_t;
use nix::Result;
use register::{Registers, Word};
use filesystem::fs::FileSystem;
use kernel::sysarg::get_sysarg_path;
use kernel::execve::shebang::expand_shebang;

pub fn translate(pid: pid_t, fs: &FileSystem, regs: &Registers) -> Result<()> {
    //	char user_path[PATH_MAX];
    //	char host_path[PATH_MAX];
    //	char new_exe[PATH_MAX];
    //	char *raw_path;
    //	const char *loader_path;
    //	int status;
    //
    //	if (IS_NOTIFICATION_PTRACED_LOAD_DONE(tracee)) {
    //		/* Syscalls can now be reported to its ptracer.  */
    //		tracee->as_ptracee.ignore_loader_syscalls = false;
    //
    //		/* Cancel this spurious kernel.execve, it was only used as a
    //		 * notification.  */
    //		set_sysnum(tracee, PR_void);
    //		return 0;
    //	}

    let user_path = get_sysarg_path(pid, regs.sys_arg_1 as *mut Word)?;
    let host_path = expand_shebang(fs, &user_path)?;

    //	if (status < 0)
    //		/* The Linux kernel actually returns -EACCES when
    //		 * trying to execute a directory.  */
    //		return status == -EISDIR ? -EACCES : status;
    //
    //	/* user_path is modified only if there's an interpreter
    //	 * (ie. for a script or with qemu).  */
    //	if (status == 0 && tracee->qemu == NULL)
    //		TALLOC_FREE(raw_path);
    //
    //	/* Remember the new value for "/proc/self/exe".  It points to
    //	 * a canonicalized guest path, hence detranslate_path()
    //	 * instead of using user_path directly.  */
    //	strcpy(new_exe, host_path);
    //	status = detranslate_path(tracee, new_exe, NULL);
    //	if (status >= 0) {
    //		talloc_unlink(tracee, tracee->new_exe);
    //		tracee->new_exe = talloc_strdup(tracee, new_exe);
    //	}
    //	else
    //		tracee->new_exe = NULL;
    //
    //	if (tracee->qemu != NULL) {
    //		status = expand_runner(tracee, host_path, user_path);
    //		if (status < 0)
    //			return status;
    //	}
    //
    //	TALLOC_FREE(tracee->load_info);
    //
    //	tracee->load_info = talloc_zero(tracee, LoadInfo);
    //	if (tracee->load_info == NULL)
    //		return -ENOMEM;
    //
    //	tracee->load_info->host_path = talloc_strdup(tracee->load_info, host_path);
    //	if (tracee->load_info->host_path == NULL)
    //		return -ENOMEM;
    //
    //	tracee->load_info->user_path = talloc_strdup(tracee->load_info, user_path);
    //	if (tracee->load_info->user_path == NULL)
    //		return -ENOMEM;
    //
    //	tracee->load_info->raw_path = (raw_path != NULL
    //			? talloc_reparent(tracee->ctx, tracee->load_info, raw_path)
    //			: talloc_reference(tracee->load_info, tracee->load_info->user_path));
    //	if (tracee->load_info->raw_path == NULL)
    //		return -ENOMEM;
    //
    //	status = extract_load_info(tracee, tracee->load_info);
    //	if (status < 0)
    //		return status;
    //
    //	if (tracee->load_info->interp != NULL) {
    //		status = extract_load_info(tracee, tracee->load_info->interp);
    //		if (status < 0)
    //			return status;
    //
    //		/* An ELF interpreter is supposed to be
    //		 * standalone.  */
    //		if (tracee->load_info->interp->interp != NULL)
    //			return -EINVAL;
    //	}
    //
    //	compute_load_addresses(tracee);
    //
    //	/* Execute the loader instead of the program.  */
    //	loader_path = get_loader_path(tracee);
    //	if (loader_path == NULL)
    //		return -ENOENT;
    //
    //	status = set_sysarg_path(tracee, loader_path, SYSARG_1);
    //	if (status < 0)
    //		return status;
    //
    //	/* Mask to its ptracer kernel performed by the loader.  */
    //	tracee->as_ptracee.ignore_loader_syscalls = true;
    //
    //	return 0;

    Ok(())
}
