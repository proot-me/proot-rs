use std::path::PathBuf;

use crate::errors::*;
use crate::filesystem::binding::Side;
use crate::process::tracee::Tracee;
use crate::register::{Current, PtraceReader, SysArg, SysArg1, SysResult};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let sys_num = tracee.regs.get_sys_num(Current);
    let absolute_guest_path = if sys_num == sc::nr::CHDIR {
        let path = tracee.regs.get_sysarg_path(SysArg1)?;
        if path.is_relative() {
            let mut guest_path = PathBuf::from(tracee.fs.borrow().get_cwd());
            guest_path.push(path);
            guest_path
        } else {
            path
        }
    } else if sys_num == sc::nr::FCHDIR {
        tracee.get_path_from_fd(
            tracee.regs.get(Current, SysArg(SysArg1)) as i32,
            Side::Guest,
        )?
    } else {
        // This check prevents us from incorrectly handling system calls other than
        // `CHDIR` and `FCHDIR`.
        Err(Error::errno_with_msg(
            Errno::ENOSYS,
            format!(
                "sysno should be CHDIR({}) or FCHDIR({}), but got {}",
                sc::nr::CHDIR,
                sc::nr::FCHDIR,
                sys_num
            ),
        ))?
    };

    tracee.fs.borrow_mut().set_cwd(absolute_guest_path)?;

    // Avoid this syscall
    tracee
        .regs
        .cancel_syscall("Cancel chdir since it is fully emulated");

    Ok(())
}

pub fn exit(tracee: &mut Tracee) -> Result<()> {
    // This syscall is fully emulated, see method `enter()` above.

    tracee
        .regs
        .set(SysResult, 0u64, "update return value in chdir::exit()");
    tracee.regs.set_restore_original_regs(false);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::test_with_proot;

    #[test]
    fn test_chdir_fchdir_and_getcwd() {
        test_with_proot(
            |tracee, _is_sysenter, _before_translation| {
                let fs = tracee.fs.borrow();
                assert!(fs.is_path_canonical(fs.get_cwd(), Side::Guest));
            },
            || {
                macro_rules! assert_with_chdir {
                    ($path:expr, $chdir_result:expr, $getcwd_result:expr) => {
                        assert_eq!(
                            nix::unistd::chdir($path).map_err(Into::<Error>::into),
                            $chdir_result
                        );
                        assert_eq!(nix::unistd::getcwd(), $getcwd_result.map(Into::into));
                    };
                }

                macro_rules! assert_with_fchdir {
                    ($path:expr, $open_result:expr, $fchdir_result:expr, $getcwd_result:expr) => {
                        let fd = nix::fcntl::open(
                            $path,
                            nix::fcntl::OFlag::O_RDONLY,
                            nix::sys::stat::Mode::empty(),
                        );
                        assert_eq!(fd.map(|_| ()).map_err(Into::<Error>::into), $open_result);
                        if fd.is_ok() {
                            // only try fchdir() after dir is opened successfully.
                            assert_eq!(
                                nix::unistd::fchdir(fd.unwrap()).map_err(Into::<Error>::into),
                                $fchdir_result
                            );
                        }
                        assert_eq!(nix::unistd::getcwd(), $getcwd_result.map(Into::into));
                    };
                }

                // the initial cwd should be "/"
                assert_eq!(nix::unistd::getcwd(), Ok("/".into()));

                // test for chdir()
                // test chdir to self
                assert_with_chdir!("/", Ok(()), Ok("/"));
                assert_with_chdir!(".", Ok(()), Ok("/"));
                // chdir to "/etc"
                assert_with_chdir!("/etc", Ok(()), Ok("/etc"));
                assert_with_chdir!("/etc/", Ok(()), Ok("/etc"));
                assert_with_chdir!(".", Ok(()), Ok("/etc"));
                // test chdir with parent dir
                assert_with_chdir!("../../../../", Ok(()), Ok("/"));
                assert_with_chdir!("../etc", Ok(()), Ok("/etc"));
                // test chdir with not exist
                assert_with_chdir!(
                    "./impossible_path",
                    Err(Error::errno(Errno::ENOENT)),
                    Ok("/etc")
                );
                assert_with_chdir!(
                    "./impossible_path/path",
                    Err(Error::errno(Errno::ENOENT)),
                    Ok("/etc")
                );

                // test for fchdir()
                // test fchdir to self
                assert_with_fchdir!("/", Ok(()), Ok(()), Ok("/"));
                assert_with_fchdir!(".", Ok(()), Ok(()), Ok("/"));
                // fchdir to "/etc"
                assert_with_fchdir!("/etc", Ok(()), Ok(()), Ok("/etc"));
                assert_with_fchdir!("/etc/", Ok(()), Ok(()), Ok("/etc"));
                assert_with_fchdir!(".", Ok(()), Ok(()), Ok("/etc"));
                // test fchdir with parent dir
                assert_with_fchdir!("../../../../", Ok(()), Ok(()), Ok("/"));
                assert_with_fchdir!("../etc", Ok(()), Ok(()), Ok("/etc"));
                // test fchdir with not exist
                assert_with_fchdir!(
                    "./impossible_path",
                    Err(Error::errno(Errno::ENOENT)),
                    Ok(()),
                    Ok("/etc")
                );
                assert_with_fchdir!(
                    "./impossible_path/path",
                    Err(Error::errno(Errno::ENOENT)),
                    Ok(()),
                    Ok("/etc")
                );
            },
        )
    }
}
