use std::os::unix::prelude::RawFd;

use nix::fcntl::AtFlags;

use crate::errors::*;
use crate::filesystem::ext::PathExt;
use crate::kernel::syscall;
use crate::register::PtraceWriter;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysArg2, SysArg3, SysArg4, SysArg5};
use crate::{errors::Result, process::tracee::Tracee};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let sys_num = tracee.regs.get_sys_num(Current);
    let dirfd = tracee.regs.get(Current, SysArg(SysArg1)) as RawFd;
    let raw_path = tracee.regs.get_sysarg_path(SysArg2)?;

    let flags_arg_index = match sys_num {
        sc::nr::FCHOWNAT | sc::nr::NAME_TO_HANDLE_AT => SysArg5,
        sc::nr::UTIMENSAT => SysArg4,
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        sc::nr::NEWFSTATAT => SysArg4,
        sc::nr::STATX => SysArg3,
        _ =>
        // This check prevents us from incorrectly handling system calls other than
        // `CHDIR` and `FCHDIR`.
        {
            Err(Error::errno_with_msg(
                Errno::ENOSYS,
                format!(
                    "An unexpected system call was received in stat_at::enter(): {}<{}>",
                    syscall::name_of_syscall(sys_num).unwrap_or("unknown"),
                    sys_num,
                ),
            ))?
        }
    };
    let flags = AtFlags::from_bits_truncate(tracee.regs.get(Current, SysArg(flags_arg_index)) as _);

    // Determines whether we need to dereference a path if it is a symbolic link.
    // Some system calls will dereference the path by default, while others do not,
    // which can also be controlled by `flags`.
    let deref_final = match sys_num {
        sc::nr::NAME_TO_HANDLE_AT => {
            flags.contains(AtFlags::AT_SYMLINK_FOLLOW) || raw_path.with_trailing_slash()
        }
        sc::nr::STATX | sc::nr::UTIMENSAT | sc::nr::FCHOWNAT => {
            !flags.contains(AtFlags::AT_SYMLINK_NOFOLLOW) || raw_path.with_trailing_slash()
        }
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        sc::nr::NEWFSTATAT => {
            !flags.contains(AtFlags::AT_SYMLINK_NOFOLLOW) || raw_path.with_trailing_slash()
        }
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
    use std::fs::File;

    use nc::file_handle_t;
    use nix::{fcntl::OFlag, sys::stat::Mode};

    use crate::utils::tests::test_with_proot;

    // TODO: reference MAX_HANDLE_SZ which is defined in <fcntl.h>. see:
    // https://elixir.bootlin.com/linux/v5.12.12/source/include/linux/exportfs.h#L15
    // https://man7.org/linux/man-pages/man2/open_by_handle_at.2.html
    const MAX_HANDLE_SZ: usize = 128;

    /// Unit test for the following syscalls:
    /// - FCHOWNAT
    /// - NAME_TO_HANDLE_AT
    /// - NEWFSTATAT
    /// - UTIMENSAT
    /// - STATX
    #[test]
    fn test_stat_at() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let filepath = "/tmp/file_for_test_stat_at";
                let filename = "file_for_test_stat_at";
                let linkpath = "/tmp/link_for_test_stat_at";
                let linkname = "link_for_test_stat_at";

                let result = std::panic::catch_unwind(|| {
                    // open "/tmp"
                    let fd = nix::fcntl::open("/tmp", OFlag::O_RDONLY, Mode::empty()).unwrap();
                    // init file and link to it
                    File::create(filepath).unwrap();
                    std::os::unix::fs::symlink(filename, linkpath).unwrap();

                    // test statx()

                    // test query file type by statx()
                    let mut statx = nc::statx_t::default();
                    // Here we need to set flags to zero, since `AT_SYMLINK_FOLLOW` is not accepted
                    // in this syscall, which will cause an `EINVAL`
                    nc::statx(fd, linkname, 0, nc::STATX_TYPE, &mut statx).unwrap();
                    // should be a regular file, since AT_SYMLINK_NOFOLLOW is not set.
                    assert_eq!((statx.stx_mode as u32 & nc::S_IFMT), nc::S_IFREG);
                    nc::statx(
                        fd,
                        linkname,
                        nc::AT_SYMLINK_NOFOLLOW,
                        nc::STATX_TYPE,
                        &mut statx,
                    )
                    .unwrap();
                    // should be a symlink, since AT_SYMLINK_NOFOLLOW is set.
                    assert_eq!((statx.stx_mode as u32 & nc::S_IFMT), nc::S_IFLNK);

                    // test newfstatat()
                    let mut stat = nc::stat_t::default();
                    nc::newfstatat(fd, linkname, &mut stat, 0).unwrap();
                    // should be a regular file, since AT_SYMLINK_NOFOLLOW is not set.
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);
                    nc::newfstatat(fd, linkname, &mut stat, nc::AT_SYMLINK_NOFOLLOW).unwrap();
                    // should be a symlink, since AT_SYMLINK_NOFOLLOW is set.
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFLNK);

                    // test utimensat()

                    // set access and modification times for the file
                    let time = [
                        nc::timespec_t {
                            tv_sec: 100,
                            tv_nsec: 0,
                        },
                        nc::timespec_t {
                            tv_sec: 101,
                            tv_nsec: 0,
                        },
                    ];
                    nc::utimensat(fd, linkname, &time, 0).unwrap();
                    // check access time and modification time for the file
                    let file_stat = nix::sys::stat::lstat(filepath).unwrap();
                    assert_eq!(file_stat.st_atime, time[0].tv_sec as _);
                    assert_eq!(file_stat.st_atime_nsec, (time[0].tv_nsec) as _);
                    assert_eq!(file_stat.st_mtime, time[1].tv_sec as _);
                    assert_eq!(file_stat.st_mtime_nsec, (time[1].tv_nsec) as _);

                    // set access and modification times for the link
                    let time = [
                        nc::timespec_t {
                            tv_sec: 200,
                            tv_nsec: 0,
                        },
                        nc::timespec_t {
                            tv_sec: 201,
                            tv_nsec: 0,
                        },
                    ];
                    nc::utimensat(fd, linkname, &time, nc::AT_SYMLINK_NOFOLLOW).unwrap();
                    // check access time and modification time for the link
                    let link_stat = nix::sys::stat::lstat(linkpath).unwrap();
                    assert_eq!(link_stat.st_atime, time[0].tv_sec as _);
                    assert_eq!(link_stat.st_atime_nsec, (time[0].tv_nsec) as _);
                    assert_eq!(link_stat.st_mtime, time[1].tv_sec as _);
                    assert_eq!(link_stat.st_mtime_nsec, (time[1].tv_nsec) as _);

                    // test name_to_handle_at()

                    // since the caller must have the CAP_DAC_READ_SEARCH capability to invoke
                    // open_by_handle_at(), we only check that the call to name_to_handle_at works.
                    let mut file_handle_buffer =
                        [0u8; MAX_HANDLE_SZ + std::mem::size_of::<file_handle_t>()];
                    let file_handle =
                        unsafe { &mut *(&mut file_handle_buffer as *mut u8 as *mut file_handle_t) };
                    // let mut file_handle = nc::file_handle_t::default();
                    let mut mount_id = 0;
                    file_handle.handle_bytes = MAX_HANDLE_SZ as _; // reset handle_bytes
                    nc::name_to_handle_at(
                        fd,
                        linkname,
                        file_handle,
                        &mut mount_id,
                        nc::AT_SYMLINK_FOLLOW,
                    )
                    .unwrap();
                    file_handle.handle_bytes = MAX_HANDLE_SZ as _; // reset handle_bytes
                    nc::name_to_handle_at(fd, linkname, file_handle, &mut mount_id, 0).unwrap();

                    // test fchownat()
                    nc::fchownat(fd, linkname, -1i64 as _, -1i64 as _, 0).unwrap();
                    nc::fchownat(
                        fd,
                        linkname,
                        -1i64 as _,
                        -1i64 as _,
                        nc::AT_SYMLINK_NOFOLLOW,
                    )
                    .unwrap();
                });
                std::fs::remove_file(filepath).unwrap();
                std::fs::remove_file(linkpath).unwrap();
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }
}
