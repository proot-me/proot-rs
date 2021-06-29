use crate::errors::*;
use crate::filesystem::Translator;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{PtraceReader, SysArg1, SysArg2};

/// Translates link and rename kernel
pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let old_path = tracee.regs.get_sysarg_path(SysArg1)?;
    let new_path = tracee.regs.get_sysarg_path(SysArg2)?;

    let old_host_path = tracee.fs.borrow().translate_path(old_path, false)?;
    let new_host_path = tracee.fs.borrow().translate_path(new_path, false)?;

    tracee.regs.set_sysarg_path(
        SysArg1,
        &old_host_path,
        "during enter open translation, setting host path",
    )?;
    tracee.regs.set_sysarg_path(
        SysArg2,
        &new_host_path,
        "during enter open translation, setting host path",
    )?;

    Ok(())
}

/// Translates `rename` and `rename_at` kernel
pub fn exit(_tracee: &mut Tracee) -> Result<()> {
    // TODO: How to track path changes of an opened file/dir?
    // We also need to change the value of field `cwd` stored in tracee, if any part
    // of cwd is modified by rename() or rename_at().

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
    // 		 * "point-of-view" as tracee->fs->cwd (guest).  */
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
    // 		 * current working directory.  */
    //		comparison = compare_paths(old_path, tracee->fs->cwd);
    //		if (comparison != PATH1_IS_PREFIX && comparison != PATHS_ARE_EQUAL) {
    //			return SyscallExitResult::Value(0);
    //		}
    //
    //		/* Get the new path, then convert it to the same
    // 		 * "point-of-view" as tracee->fs->cwd (guest).  */
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use nix::{fcntl::OFlag, sys::stat::Mode};

    use crate::utils::tests::test_with_proot;

    /// Unit test for the following syscalls:
    /// - link
    /// - rename
    /// - renameat
    #[test]
    fn test_link_rename() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let original_filepath = "/tmp/original_filepath_for_test_link_rename";
                let original_linkpath = "/tmp/original_linkpath_for_test_link_rename";
                let cloned_linkpath = "/tmp/cloned_linkpath_for_test_link_rename";
                let cloned_filepath = "/tmp/cloned_filepath_for_test_link_rename";
                let renamed_filepath = "/tmp/renamed_filepath_for_test_link_rename";
                let renamed_filename = "renamed_filepath_for_test_link_rename";
                let rerenamed_filename = "re-renamed_filepath_for_test_link_rename";
                let rerenamed_filepath = "/tmp/re-renamed_filepath_for_test_link_rename";

                let result = std::panic::catch_unwind(|| {
                    // open "/tmp"
                    let fd = nix::fcntl::open("/tmp", OFlag::O_RDONLY, Mode::empty()).unwrap();
                    // init file
                    File::create(original_filepath).unwrap();
                    std::os::unix::fs::symlink(original_filepath, original_linkpath).unwrap();

                    // test link()

                    // This will clone the original symbolic link, because link() does not
                    // dereference the symbolic link
                    nc::link(original_linkpath, cloned_linkpath).unwrap();
                    let mut stat = nc::stat_t::default();
                    nc::lstat(cloned_linkpath, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFLNK);
                    let mut buf = [0_u8; nc::PATH_MAX as usize];
                    let n_read = nc::readlink(cloned_linkpath, &mut buf).unwrap() as usize;
                    assert_eq!(original_filepath.as_bytes(), &buf[0..n_read]);

                    // This will make a hard link to the `original_filepath`
                    nc::link(original_filepath, cloned_filepath).unwrap();
                    let mut cloned_filestat = nc::stat_t::default();
                    nc::lstat(cloned_filepath, &mut cloned_filestat).unwrap();
                    assert_eq!((cloned_filestat.st_mode & nc::S_IFMT), nc::S_IFREG);

                    let mut original_filestat = nc::stat_t::default();
                    nc::lstat(original_filepath, &mut original_filestat).unwrap();
                    assert_eq!(cloned_filestat.st_ino, original_filestat.st_ino);

                    // test rename()

                    nc::rename(cloned_filepath, renamed_filepath).unwrap();
                    let mut stat = nc::stat_t::default();
                    // This file does not exist because it has been renamed.
                    assert_eq!(nc::lstat(cloned_filepath, &mut stat), Err(nc::ENOENT));
                    nc::lstat(renamed_filepath, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);

                    // test renameat()
                    nc::renameat(fd, renamed_filename, fd, rerenamed_filename).unwrap();
                    let mut stat = nc::stat_t::default();
                    // This file does not exist because it has been renamed.
                    assert_eq!(nc::lstat(renamed_filepath, &mut stat), Err(nc::ENOENT));
                    nc::lstat(rerenamed_filepath, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);
                });

                let _ = std::fs::remove_file(original_filepath);
                let _ = std::fs::remove_file(original_linkpath);
                let _ = std::fs::remove_file(cloned_linkpath);
                let _ = std::fs::remove_file(cloned_filepath);
                let _ = std::fs::remove_file(renamed_filepath);
                let _ = std::fs::remove_file(rerenamed_filepath);
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }
}
