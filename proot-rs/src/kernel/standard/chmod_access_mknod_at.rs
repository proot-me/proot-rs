use std::os::unix::prelude::RawFd;

use crate::errors::*;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysArg2};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let sys_num = tracee.regs.get_sys_num(Current);
    let dirfd = tracee.regs.get(Current, SysArg(SysArg1)) as RawFd;
    let raw_path = tracee.regs.get_sysarg_path(SysArg2)?;

    let deref_final = match sys_num {
        sc::nr::MKNODAT => false, /* By default, mknodat() will not follow a symbolic link. https://man7.org/linux/man-pages/man2/mknod.2.html */
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm"))]
        sc::nr::FUTIMESAT => true,
        sc::nr::FACCESSAT | sc::nr::FCHMODAT => true,
        _ => true,
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
    use std::os::unix::prelude::PermissionsExt;

    use nix::{fcntl::OFlag, sys::stat::Mode};

    use crate::utils::tests::test_with_proot;

    /// Unit test for the following syscalls:
    /// - FCHMODAT
    /// - FACCESSAT
    /// - FUTIMESAT
    /// - MKNODAT
    #[test]
    fn test_chmod_access_mknod_at() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let filepath = "/tmp/file_for_unit_test";
                let filename = "file_for_unit_test";

                let result = std::panic::catch_unwind(|| {
                    // open "/tmp"
                    let fd = nix::fcntl::open("/tmp", OFlag::O_RDONLY, Mode::empty()).unwrap();

                    // test mknodat()

                    // call mknodat(fd, filename) to create a regular file.
                    nc::mknodat(
                        fd,
                        filename,
                        nc::S_IFREG
                            | nc::S_IRUSR
                            | nc::S_IWUSR
                            | nc::S_IRGRP
                            | nc::S_IWGRP
                            | nc::S_IROTH
                            | nc::S_IWOTH,
                        0,
                    )
                    .unwrap();
                    // check file was created.
                    let metadata = std::fs::metadata(filepath).unwrap();
                    assert!(metadata.is_file());

                    // test fchmodat() and faccessat()

                    // call fchmodat() on this file to set mode to "700", and check mode is changed
                    // successfully.
                    nc::fchmodat(fd, filename, 0o700).unwrap();
                    let metadata = std::fs::metadata(filepath).unwrap();
                    let mode = metadata.permissions().mode();
                    assert_eq!(
                        Mode::from_bits_truncate(mode),
                        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IXUSR
                    );
                    // since mode is "700", we have full access to this file
                    nc::faccessat(fd, filename, nc::F_OK | nc::R_OK | nc::W_OK | nc::X_OK).unwrap();

                    // call fchmodat() on this file to set mode to "000", and check it later
                    nc::fchmodat(fd, filename, 0o000).unwrap();
                    let metadata = std::fs::metadata(filepath).unwrap();
                    let mode = metadata.permissions().mode();
                    assert_eq!(Mode::from_bits_truncate(mode), Mode::empty());
                    // mode is changed to "000", so we should have no access to this file
                    nc::faccessat(fd, filename, nc::F_OK | nc::R_OK | nc::W_OK | nc::X_OK)
                        .unwrap_err();

                    // test futimesat()

                    // set access and modification times for this file
                    let time = [
                        nc::timeval_t {
                            tv_sec: 100,
                            tv_usec: 0,
                        },
                        nc::timeval_t {
                            tv_sec: 10,
                            tv_usec: 0,
                        },
                    ];
                    nc::futimesat(fd, filename, &time).unwrap();
                    // check access time and modification time
                    let file_stat = nix::sys::stat::stat(filepath).unwrap();
                    assert_eq!(file_stat.st_atime, time[0].tv_sec as _);
                    assert_eq!(file_stat.st_atime_nsec, (time[0].tv_usec * 1000) as _);
                    assert_eq!(file_stat.st_mtime, time[1].tv_sec as _);
                    assert_eq!(file_stat.st_mtime_nsec, (time[1].tv_usec * 1000) as _);
                });
                std::fs::remove_file(filepath).unwrap();
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }
}
