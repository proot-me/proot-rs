use crate::errors::*;

use crate::filesystem::Translator;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{PtraceReader, SysArg1};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let raw_path = tracee.regs.get_sysarg_path(SysArg1)?;

    let host_path = tracee.fs.borrow().translate_path(raw_path, true)?;

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

    /// Unit test for all the standard syscalls:
    /// access, acct, chmod, chown, chroot, getxattr, listxattr, mknod, creat,
    /// removexattr, setxattr, stat, swapoff, swapon, truncate, umount2, uselib,
    /// utime, utimes
    ///
    /// Since the arguments of those syscalls follow a certain pattern, only the
    /// stat() call is tested in our unit tests.
    #[test]
    fn test_standard_syscall() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let filepath = "/tmp/file_for_test_standard_syscall";
                let linkpath = "/tmp/link_for_test_standard_syscall";

                let result = std::panic::catch_unwind(|| {
                    // init file and symlink file
                    File::create(filepath).unwrap();
                    std::os::unix::fs::symlink(filepath, linkpath).unwrap();

                    // test stat()

                    let mut stat = nc::stat_t::default();
                    nc::stat(linkpath, &mut stat).unwrap();
                    // should be a regular file, since symbol link file will be dereference
                    // automatically.
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);
                    nc::stat(filepath, &mut stat).unwrap();
                    assert_eq!((stat.st_mode & nc::S_IFMT), nc::S_IFREG);
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
