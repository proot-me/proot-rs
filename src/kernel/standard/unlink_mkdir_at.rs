use std::os::unix::prelude::RawFd;

use crate::errors::*;
use crate::filesystem::ext::PathExt;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysArg2};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let sys_num = tracee.regs.get_sys_num(Current);
    let dirfd = tracee.regs.get(Current, SysArg(SysArg1)) as RawFd;
    let raw_path = tracee.regs.get_sysarg_path(SysArg2)?;

    let deref_final = match sys_num {
        sc::nr::UNLINKAT | sc::nr::MKDIRAT => false,
        _ => raw_path.with_trailing_slash(),
    };

    let host_path = tracee.translate_path_at(dirfd, raw_path, deref_final)?.1;

    tracee.regs.set_sysarg_path(
        SysArg2,
        &host_path,
        "during enter open translation, setting host path",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use nix::{fcntl::OFlag, sys::stat::Mode};

    use crate::utils::tests::test_with_proot;

    /// Unit test for the following syscalls:
    /// - unlinkat
    /// - mkdirat
    /// - readlinkat
    #[test]
    fn test_unlink_mkdir_at() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let dirpath = "/tmp/dir_for_test_unlink_mkdir_at";
                let dirname = "dir_for_test_unlink_mkdir_at";
                let linkpath = "/tmp/link_for_test_unlink_mkdir_at";
                let linkname = "link_for_test_unlink_mkdir_at";

                let result = std::panic::catch_unwind(|| {
                    // open "/tmp"
                    let fd = nix::fcntl::open("/tmp", OFlag::O_RDONLY, Mode::empty()).unwrap();

                    //test mkdirat()

                    // create a directory, and a symbolic link file with pointing to the previous
                    // directory.
                    nc::symlink(dirpath, linkpath).unwrap();
                    // should be failed, because linkpath is already here
                    assert_eq!(nc::mkdirat(fd, linkname, 0o755), Err(nc::EEXIST));
                    nc::mkdirat(fd, dirname, 0o755).unwrap();

                    // test readlinkat()

                    // call readlinkat() with `linkname`
                    let mut buf = [0u8; nc::PATH_MAX as usize];
                    let n_read = nc::readlinkat(fd, linkname, &mut buf).unwrap() as usize;
                    assert_eq!(n_read, dirpath.len());
                    assert_eq!(dirpath.as_bytes(), &buf[0..n_read]);
                    // call readlinkat() with `dirname`
                    assert_eq!(nc::readlinkat(fd, dirname, &mut buf), Err(nc::EINVAL));

                    // test unlinkat()

                    // should fail when unlinkat() a symlink file with nc::AT_REMOVEDIR flag set
                    assert_eq!(
                        nc::unlinkat(fd, linkname, nc::AT_REMOVEDIR),
                        Err(nc::ENOTDIR)
                    );
                    nc::unlinkat(fd, linkname, 0).unwrap();
                    // should fail when unlinkat() a directory without a nc::AT_REMOVEDIR flag
                    assert_eq!(nc::unlinkat(fd, dirname, 0), Err(nc::EISDIR));
                    nc::unlinkat(fd, dirname, nc::AT_REMOVEDIR).unwrap();
                });

                let _ = std::fs::remove_file(linkpath);
                let _ = std::fs::remove_dir(dirpath);
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }
}
