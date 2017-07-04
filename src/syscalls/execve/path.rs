use nix::Result;

/// Translate 'user_path' into 'host_path' and check if this latter exists, is
/// executable and is a regular file.  This function returns -errno if
/// an error occurred, 0 otherwise.
// int translate_and_check_exec(Tracee *tracee, char host_path[PATH_MAX], const char *user_path)
pub fn translate_and_check_exec() -> Result<()> {
//	struct stat statl;
//	int status;
//
//	if (user_path[0] == '\0')
//		return -ENOEXEC;
//
    

//	status = translate_path(tracee, host_path, AT_FDCWD, user_path, true);
//	if (status < 0)
//		return status;
//
//	status = access(host_path, F_OK);
//	if (status < 0)
//		return -ENOENT;
//
//	status = access(host_path, X_OK);
//	if (status < 0)
//		return -EACCES;
//
//	status = lstat(host_path, &statl);
//	if (status < 0)
//		return -EPERM;
//
//	return 0;

    Ok(())
}