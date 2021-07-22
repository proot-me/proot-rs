use nix::fcntl::OFlag;

use crate::errors::*;

use crate::filesystem::Translator;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysArg2};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let raw_path = tracee.regs.get_sysarg_path(SysArg1)?;

    let flags = OFlag::from_bits_truncate(tracee.regs.get(Current, SysArg(SysArg2)) as _);

    let deref_final = !(flags.contains(OFlag::O_NOFOLLOW)
        || (flags.contains(OFlag::O_EXCL) && flags.contains(OFlag::O_CREAT)));
    let host_path = tracee.fs.borrow().translate_path(raw_path, deref_final)?;

    tracee.regs.set_sysarg_path(
        SysArg1,
        &host_path,
        "during enter open translation, setting host path",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::utils::tests::test_with_proot;

    /// Unit test for the following syscalls:
    /// - linkat
    #[test]
    fn test_open() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let filepath = "/tmp/file_for_test_open";
                let linkpath = "/tmp/link_for_test_open";

                let result = std::panic::catch_unwind(|| {
                    // init symlink
                    std::os::unix::fs::symlink(filepath, linkpath).unwrap();

                    // Test open(linkpath + "/") with `O_CREAT` and `O_EXCL`, and this will get a
                    // EISDIR, and symlink follow didn't not happen.
                    assert_eq!(
                        nc::open(
                            format!("{}/", linkpath).as_str(),
                            nc::O_RDONLY | nc::O_CREAT | nc::O_EXCL,
                            0o755
                        ),
                        Err(nc::EISDIR)
                    );

                    // Test open(linkpath) with `O_CREAT` and `O_EXCL`, and this will get a EEXIST,
                    // because symlink follow didn't not happen.
                    assert_eq!(
                        nc::open(linkpath, nc::O_RDONLY | nc::O_CREAT | nc::O_EXCL, 0o755),
                        Err(nc::EEXIST)
                    );

                    // Test open(linkpath) with `O_CREAT`, and this will create a regular file at
                    // `filepath`, because symlink follow happened.
                    let file_fd = nc::open(linkpath, nc::O_RDONLY | nc::O_CREAT, 0o755).unwrap();
                    let mut stat = nc::stat_t::default();
                    nc::fstat(file_fd, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);
                    nc::unlink(filepath).unwrap();

                    // Test open(filepath) with `O_CREAT` and `O_EXCL`, and this will create a
                    // regular file at `filepath`.
                    let file_fd =
                        nc::open(filepath, nc::O_RDONLY | nc::O_CREAT | nc::O_EXCL, 0o755).unwrap();
                    let mut stat = nc::stat_t::default();
                    nc::fstat(file_fd, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);
                    nc::close(file_fd).unwrap();

                    // test open() with `OFlag::O_NOFOLLOW`;
                    let file_fd = nc::open(linkpath, nc::O_NOFOLLOW | nc::O_PATH, 0).unwrap();
                    let mut stat = nc::stat_t::default();
                    nc::fstat(file_fd, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFLNK);
                    nc::close(file_fd).unwrap();

                    // test open() in normal case;
                    let file_fd = nc::open(linkpath, 0, 0).unwrap();
                    let mut stat = nc::stat_t::default();
                    nc::fstat(file_fd, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);
                    nc::close(file_fd).unwrap();
                });
                let _ = std::fs::remove_file(linkpath);
                let _ = std::fs::remove_file(filepath);
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }
}
