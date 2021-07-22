use std::os::unix::prelude::RawFd;

use crate::errors::*;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg2, SysArg3};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let dirfd = tracee.regs.get(Current, SysArg(SysArg2)) as RawFd;
    let raw_path = tracee.regs.get_sysarg_path(SysArg3)?;

    // create/delete/rename related system calls cannot follow final component.
    let host_path = tracee.translate_path_at(dirfd, raw_path, false)?;

    tracee.regs.set_sysarg_path(
        SysArg3,
        &host_path,
        "during enter open translation, setting host path",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use nix::{fcntl::OFlag, sys::stat::Mode};

    use crate::utils::tests::test_with_proot;

    /// Unit test for the following syscalls:
    /// - symlinkat
    #[test]
    fn test_sym_link_at() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let filepath = "/tmp/file_for_test_sym_link_at";
                let filename = "file_for_test_sym_link_at";
                let linkpath_1 = "/tmp/link_1_for_test_sym_link_at";
                let linkpath_2 = "/tmp/link_2_for_test_sym_link_at";

                let result = std::panic::catch_unwind(|| {
                    // open "/tmp"
                    let fd = nix::fcntl::open("/tmp", OFlag::O_RDONLY, Mode::empty()).unwrap();

                    // create two symbolic link file with symlinkat(), one points to the absolute
                    // path and one to the relative path.
                    File::create(filepath).unwrap();
                    nc::symlinkat(filename, fd, linkpath_1).unwrap();
                    nc::symlinkat(filepath, fd, linkpath_2).unwrap();

                    // check correctness of symlinkat() by examining the resulting symbolic link
                    // file.
                    let mut stat = nc::stat_t::default();
                    // both `linkpath_1` and `linkpath_2` should be symlink
                    nc::lstat(linkpath_1, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFLNK);
                    nc::lstat(linkpath_2, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFLNK);
                    // both `linkpath_1` and `linkpath_2` should point to a regular file
                    nc::stat(linkpath_1, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);
                    nc::stat(linkpath_2, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);
                });
                std::fs::remove_file(filepath).unwrap();
                std::fs::remove_file(linkpath_1).unwrap();
                std::fs::remove_file(linkpath_2).unwrap();
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }
}
