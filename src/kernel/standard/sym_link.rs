use crate::errors::*;

use crate::filesystem::Translator;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::{PtraceReader, SysArg2};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    let raw_path = tracee.regs.get_sysarg_path(SysArg2)?;
    let host_path = tracee.fs.borrow().translate_path(raw_path, false)?;

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

    use crate::utils::tests::test_with_proot;

    /// Unit test for the following syscalls:
    /// - symlink
    #[test]
    fn test_sym_link() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let filepath = "/tmp/file_for_test_sym_link";
                let filename = "file_for_test_sym_link";
                let linkpath_1 = "/tmp/link_1_for_test_sym_link";
                let linkpath_2 = "/tmp/link_2_for_test_sym_link";

                let result = std::panic::catch_unwind(|| {
                    // create two symbolic link file, one points to the absolute path and one to the
                    // relative path.
                    File::create(filepath).unwrap();
                    nc::symlink(filename, linkpath_1).unwrap();
                    nc::symlink(filepath, linkpath_2).unwrap();

                    // check correctness of symlink() by examining the resulting symbolic link file.
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
