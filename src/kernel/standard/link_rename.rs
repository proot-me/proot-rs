use crate::errors::Result;
use crate::kernel::exit::SyscallExitResult;

/// Translates link and rename kernel
pub fn enter() -> Result<()> {
    Ok(())

    //    status = translate_sysarg(tracee, SYSARG_1, SYMLINK);
    //    if (status < 0)
    //        break;
    //
    //    status = translate_sysarg(tracee, SYSARG_2, SYMLINK);
}

/// Translates `rename` and `rename_at` kernel
pub fn exit() -> SyscallExitResult {
    //    char old_path[PATH_MAX];
    //		char new_path[PATH_MAX];
    //		ssize_t old_length;
    //		ssize_t new_length;
    //		Comparison comparison;
    //		Reg old_reg;
    //		Reg new_reg;
    //		char *tmp;
    //
    //		/* Error reported by the kernel.  */
    //		if ((int) syscall_result < 0)
    //			return SyscallExitResult::None;
    //
    //		if (syscall_number == PR_rename) {
    //			old_reg = SYSARG_1;
    //			new_reg = SYSARG_2;
    //		}
    //		else {
    //			old_reg = SYSARG_2;
    //			new_reg = SYSARG_4;
    //		}
    //
    //		/* Get the old path, then convert it to the same
    //		 * "point-of-view" as tracee->fs->cwd (guest).  */
    //		status = read_path(tracee, old_path, peek_reg(tracee, MODIFIED, old_reg));
    //		if (status < 0)
    //			return SyscallExitResult::Value(status);
    //
    //		status = detranslate_path(tracee, old_path, NULL);
    //		if (status < 0)
    //			return SyscallExitResult::Value(status);
    //		old_length = (status > 0 ? status - 1 : (ssize_t) strlen(old_path));
    //
    //		/* Nothing special to do if the moved path is not the
    //		 * current working directory.  */
    //		comparison = compare_paths(old_path, tracee->fs->cwd);
    //		if (comparison != PATH1_IS_PREFIX && comparison != PATHS_ARE_EQUAL) {
    //			return SyscallExitResult::Value(0);
    //		}
    //
    //		/* Get the new path, then convert it to the same
    //		 * "point-of-view" as tracee->fs->cwd (guest).  */
    //		status = read_path(tracee, new_path, peek_reg(tracee, MODIFIED, new_reg));
    //		if (status < 0)
    //			return SyscallExitResult::Value(status);
    //
    //		status = detranslate_path(tracee, new_path, NULL);
    //		if (status < 0)
    //			return SyscallExitResult::Value(status);
    //		new_length = (status > 0 ? status - 1 : (ssize_t) strlen(new_path));
    //
    //		/* Sanity check.  */
    //		if (strlen(tracee->fs->cwd) >= PATH_MAX) {
    //			return SyscallExitResult::Value(0);
    //		}
    //		strcpy(old_path, tracee->fs->cwd);
    //
    //		/* Update the virtual current working directory.  */
    //		substitute_path_prefix(old_path, old_length, new_path, new_length);
    //
    //		tmp = talloc_strdup(tracee->fs, old_path);
    //		if (tmp == NULL) {
    //			return SyscallExitResult::Value(-ENOMEM);
    //		}
    //
    //		TALLOC_FREE(tracee->fs->cwd);
    //		tracee->fs->cwd = tmp;
    //
    //		status = 0;
    SyscallExitResult::None
}
