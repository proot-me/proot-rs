use crate::errors::*;

use crate::filesystem::ext::PathExt;
use crate::filesystem::Translator;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg1};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let sys_num = tracee.regs.get_sys_num(Current);
    let raw_path = tracee.regs.get_sysarg_path(SysArg1)?;

    let deref_final = match sys_num {
        // First, create/delete/rename related system calls cannot follow final component.
        sc::nr::UNLINK | sc::nr::RMDIR | sc::nr::MKDIR => false,
        _ => {
            // Second, since there is no flags here, skip check for LOOKUP_FOLLOW.
            // Third, follow if the pathname has trailing slashes.
            if raw_path.with_trailing_slash() {
                true
            } else {
                // Default value for those syscalls
                false
            }
        }
    };
    let host_path = tracee.fs.borrow().translate_path(raw_path, deref_final)?.1;

    tracee.regs.set_sysarg_path(
        SysArg1,
        &host_path,
        "during enter open translation, setting host path",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::utils::tests::test_with_proot;

    /// Unit test for the following syscalls:
    /// - lsetxattr
    /// - lgetxattr
    /// - llistxattr
    /// - lremovexattr
    /// - lstat
    /// - lchown
    /// - readlink
    /// - unlink
    /// - rmdir
    /// - mkdir
    #[test]
    fn test_dir_link_attr() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let filepath = "/tmp/file_for_test_dir_link_attr";
                let dirpath = "/tmp/dir_for_test_dir_link_attr";
                let linkpath = "/tmp/link_for_test_dir_link_attr";

                let result = std::panic::catch_unwind(|| {
                    // create a regular file and a directory, and a symbolic link file pointing
                    // to the previous regular file.
                    File::create(filepath).unwrap();
                    nc::symlink(filepath, linkpath).unwrap();

                    // test mkdir()
                    nc::mkdir(dirpath, 0o755).unwrap();

                    // test rmdir()
                    nc::rmdir(dirpath).unwrap();

                    // test lsetxattr() and lgetxattr()

                    // create an attr and read it with lsetxattr()
                    let attr_name = "user.proot-rs-unit-test";
                    let attr_value_1 = "file";
                    let attr_value_2 = "symlink";
                    // Try to set xattr for `linkpath`.
                    // This would be failed with `EPERM`, since "user extended attributes are
                    // allowed only for regular files and directories". See https://man7.org/linux/man-pages/man7/xattr.7.html
                    assert_eq!(
                        nc::lsetxattr(
                            linkpath,
                            &attr_name,
                            attr_value_1.as_ptr() as usize,
                            attr_value_1.len(),
                            0,
                        ),
                        Err(nc::EPERM)
                    );
                    // set xattr for `filepath`
                    nc::lsetxattr(
                        filepath,
                        &attr_name,
                        attr_value_2.as_ptr() as usize,
                        attr_value_2.len(),
                        0,
                    )
                    .unwrap();
                    // query with lgetxattr()
                    let mut buf = [0_u8; 16];
                    // Should be failed, because we did not successfully set attributes for
                    // `linkpath`.
                    assert_eq!(
                        nc::lgetxattr(linkpath, attr_name, buf.as_mut_ptr() as usize, buf.len()),
                        Err(nc::ENODATA)
                    );
                    nc::lgetxattr(filepath, attr_name, buf.as_mut_ptr() as usize, buf.len())
                        .unwrap();
                    assert_eq!(attr_value_2.as_bytes(), &buf[..attr_value_2.len()]);

                    // test llistxattr()

                    let mut buf = [0_u8; 128];
                    // list all xattr names with llistxattr()
                    let attr_len = nc::llistxattr(linkpath, buf.as_mut_ptr() as usize, buf.len())
                        .unwrap() as usize;
                    // `attr_len` should be `0` because there are no attributes.
                    assert_eq!(attr_len, 0);
                    let attr_len = nc::llistxattr(filepath, buf.as_mut_ptr() as usize, buf.len())
                        .unwrap() as usize;
                    assert_eq!(&buf[..attr_len - 1], attr_name.as_bytes());

                    // test lremovexattr()

                    assert_eq!(nc::lremovexattr(linkpath, attr_name), Err(nc::EPERM));
                    nc::lremovexattr(filepath, attr_name).unwrap();

                    // test lstat()

                    let mut stat = nc::stat_t::default();
                    nc::lstat(linkpath, &mut stat).unwrap();
                    // should be a symlink.
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFLNK);
                    nc::lstat(filepath, &mut stat).unwrap();
                    // should be a symlink.
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);

                    // test lchown()

                    // Since we may have no CAP_CHOWN/CAP_CHOWN capability to change owner/group of
                    // a file, we set `-1` here to indicate that we do not make changes.
                    nc::lchown(linkpath, -1i64 as _, -1i64 as _).unwrap();
                    nc::lchown(filepath, -1i64 as _, -1i64 as _).unwrap();

                    // test readlink()

                    let mut buf = [0_u8; nc::PATH_MAX as usize];
                    let n_read = nc::readlink(linkpath, &mut buf).unwrap() as usize;
                    assert_eq!(filepath.as_bytes(), &buf[0..n_read]);
                    assert_eq!(nc::readlink(filepath, &mut buf), Err(nc::EINVAL));

                    // test unlink()

                    nc::unlink(filepath).unwrap();
                    nc::unlink(linkpath).unwrap();
                });

                let _ = std::fs::remove_file(filepath);
                let _ = std::fs::remove_file(linkpath);
                let _ = std::fs::remove_dir(dirpath);
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }
}
