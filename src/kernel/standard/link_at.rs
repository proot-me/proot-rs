use std::os::unix::prelude::RawFd;

use nix::fcntl::AtFlags;

use crate::errors::*;
use crate::filesystem::ext::PathExt;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysArg2, SysArg3, SysArg4, SysArg5};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let olddirfd = tracee.regs.get(Current, SysArg(SysArg1)) as RawFd;
    let newdirfd = tracee.regs.get(Current, SysArg(SysArg3)) as RawFd;
    let old_path = tracee.regs.get_sysarg_path(SysArg2)?;
    let new_path = tracee.regs.get_sysarg_path(SysArg4)?;

    let flags = AtFlags::from_bits_truncate(tracee.regs.get(Current, SysArg(SysArg5)) as _);
    let deref_final = flags.contains(AtFlags::AT_SYMLINK_FOLLOW) || old_path.with_trailing_slash();

    let old_host_path = tracee.translate_path_at(olddirfd, old_path, deref_final)?.1;
    let new_host_path = tracee.translate_path_at(newdirfd, new_path, false)?.1;

    tracee.regs.set_sysarg_path(
        SysArg2,
        &old_host_path,
        "during enter open translation, setting host path",
    )?;
    tracee.regs.set_sysarg_path(
        SysArg4,
        &new_host_path,
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
    /// - linkat
    #[test]
    fn test_link_at() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let oldfilepath = "/tmp/old_file_for_test_link_at";
                let newfilepath = "/tmp/new_file_for_test_link_at";
                let newfilename = "new_file_for_test_link_at";

                let oldlinkpath = "/tmp/old_link_for_test_link_at";
                let oldlinkname = "old_link_for_test_link_at";
                let newlinkpath = "/tmp/new_link_for_test_link_at";
                let newlinkname = "new_link_for_test_link_at";

                let result = std::panic::catch_unwind(|| {
                    // open "/tmp"
                    let fd = nix::fcntl::open("/tmp", OFlag::O_RDONLY, Mode::empty()).unwrap();
                    // init file
                    File::create(oldfilepath).unwrap();
                    std::os::unix::fs::symlink(oldfilepath, oldlinkpath).unwrap();

                    // test linkat()

                    // This will create a new hard link to `oldlinkpath`, which means that we will
                    // create a new symbolic link.
                    nc::linkat(fd, oldlinkname, fd, newlinkname, 0).unwrap();
                    let mut stat = nc::stat_t::default();
                    nc::lstat(newlinkpath, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFLNK);
                    let mut buf = [0_u8; nc::PATH_MAX as usize];
                    let n_read = nc::readlink(newlinkpath, &mut buf).unwrap() as usize;
                    assert_eq!(oldfilepath.as_bytes(), &buf[0..n_read]);

                    // With `AT_SYMLINK_FOLLOW`, linkat() will create a hard link to the
                    // `oldfilepath` file.
                    nc::linkat(fd, oldlinkname, fd, newfilename, nc::AT_SYMLINK_FOLLOW).unwrap();
                    let mut new_filestat = nc::stat_t::default();
                    nc::lstat(newfilepath, &mut new_filestat).unwrap();
                    assert_eq!((new_filestat.st_mode & nc::S_IFMT), nc::S_IFREG);
                    let mut old_filestat = nc::stat_t::default();
                    nc::lstat(oldfilepath, &mut old_filestat).unwrap();
                    assert_eq!(new_filestat.st_ino, old_filestat.st_ino);

                    // If the oldfilename end with a trailing slash, symlink
                    // follow will also happen.
                    assert_eq!(
                        nc::linkat(fd, format!("{}/", oldlinkname).as_str(), fd, newfilename, 0),
                        Err(nc::ENOTDIR)
                    );
                });
                std::fs::remove_file(oldfilepath).unwrap();
                std::fs::remove_file(oldlinkpath).unwrap();
                std::fs::remove_file(newlinkpath).unwrap();
                std::fs::remove_file(newfilepath).unwrap();
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }
}
