use std::os::unix::prelude::RawFd;

use nix::fcntl::OFlag;

use crate::errors::*;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysArg2, SysArg3};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let dirfd = tracee.regs.get(Current, SysArg(SysArg1)) as RawFd;
    let raw_path = tracee.regs.get_sysarg_path(SysArg2)?;
    let flags = OFlag::from_bits_truncate(tracee.regs.get(Current, SysArg(SysArg3)) as _);

    let deref_final = !(flags.contains(OFlag::O_NOFOLLOW)
        || (flags.contains(OFlag::O_EXCL) && flags.contains(OFlag::O_CREAT)));

    let host_path = tracee.translate_path_at(dirfd, raw_path, deref_final)?;

    tracee.regs.set_sysarg_path(
        SysArg2,
        &host_path,
        "during enter open translation, setting host path",
    )?;
    // We don't need to modify SysArg1 because the SysArg2 is an absolute path now

    Ok(())
}

#[cfg(test)]
mod tests {
    use nix::{fcntl::OFlag, sys::stat::Mode};

    use crate::utils::tests::test_with_proot;

    /// Unit test for the following syscalls:
    /// - linkat
    #[test]
    fn test_open_at() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let filepath = "/tmp/file_for_test_open_at";
                let filename = "file_for_test_open_at";
                let linkpath = "/tmp/link_for_test_open_at";
                let linkname = "link_for_test_open_at";

                let result = std::panic::catch_unwind(|| {
                    // open "/tmp"
                    let fd = nix::fcntl::open("/tmp", OFlag::O_RDONLY, Mode::empty()).unwrap();
                    // init symlink
                    std::os::unix::fs::symlink(filepath, linkpath).unwrap();

                    // test openat()

                    // test openat() with `O_CREAT` and `O_EXCL`
                    // this will create a regular file at `filepath`
                    let file_fd =
                        nc::openat(fd, filename, nc::O_RDONLY | nc::O_CREAT | nc::O_EXCL, 0o755)
                            .unwrap();
                    let mut stat = nc::stat_t::default();
                    nc::fstat(file_fd, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);
                    nc::close(file_fd).unwrap();

                    // test openat() with `OFlag::O_NOFOLLOW`;
                    let file_fd = nc::openat(fd, linkname, nc::O_NOFOLLOW | nc::O_PATH, 0).unwrap();
                    let mut stat = nc::stat_t::default();
                    nc::fstat(file_fd, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFLNK);
                    nc::close(file_fd).unwrap();

                    // test openat() in normal case;
                    let file_fd = nc::openat(fd, linkname, 0, 0).unwrap();
                    let mut stat = nc::stat_t::default();
                    nc::fstat(file_fd, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);
                    nc::close(file_fd).unwrap();
                });
                std::fs::remove_file(linkpath).unwrap();
                std::fs::remove_file(filepath).unwrap();
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }
}
