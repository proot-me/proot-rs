//! This module is used to help to solve the trailing slash problem.
//!
//! For most syscalls (except mkdir() and rmdir() and something else), a
//! trailing slash cannot be ignored, because it causes two side effects:
//! 1. The kernel will assuming that the last component of the path is a
//! directory or a symbol link to a directory.
//! 2. If the path is a symbol link file, the kernel will follow the link
//! file recursively.
//! We only need to deal with the second case, the first case is left to the
//! kernel to deal with. That is, adding a possibility to make the value of
//! `deref_final` be `true`: when there is a trailing slash at the end of the
//! path.
//!
//! As mentioned in docs of [`PathBuf::components()`], the rust std library will
//! preform a small amount of normalization on path in some case. This
//! unperceived behavior brings some imperceptible effects to the proot-rs'
//! behavior. However rust lacks functions for handling trailing slash. So we
//! extended `PathBuf` and `AsRef<Path>` to meet our requirements.
//!
//! Note: Due to the rule of normalization in rust PathBuf, both the trailing
//! "/" and trailing "/." are ignored. Since the two mean almost the same thing,
//! they are both considered to be "trailing slash" in proot-rs.
//!
//! [`PathBuf::components()`]: https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.components

use std::{
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};

use nix::NixPath;

pub trait PathExt {
    /// Check if this path contains trailing slash (i.e. ends with "/" or "/.").
    fn with_trailing_slash(&self) -> bool;
}

impl<T> PathExt for T
where
    T: AsRef<Path>,
{
    fn with_trailing_slash(&self) -> bool {
        let bytes = self.as_ref().as_os_str().as_bytes();
        let len = bytes.len();
        (len >= 1 && bytes.get(len - 1) == Some(&b'/'))
            || (len >= 2 && bytes.get(len - 2) == Some(&b'/') && bytes.get(len - 1) == Some(&b'.'))
    }
}

pub trait PathBufExt {
    fn try_add_trailing_slash(&mut self);
}

impl PathBufExt for PathBuf {
    /// Add a trailing slash ("/") to PathBuf.
    /// You can't assume that `self` will eventually ends with "/", since it
    /// could also be ends with "/.". Furthermore, if self is empty path (""),
    /// then no changes will be made to `self`.
    fn try_add_trailing_slash(&mut self) {
        if !self.with_trailing_slash() && !self.is_empty() {
            unsafe {
                let old_self = std::ptr::read(self);
                let mut os_string = old_self.into_os_string();
                os_string.push("/");
                let new_self = PathBuf::from(os_string);
                std::ptr::write(self, new_self);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::utils::tests::test_with_proot;

    use super::*;

    #[test]
    fn test_with_trailing_slash() {
        assert_eq!("".with_trailing_slash(), false);
        assert_eq!("/".with_trailing_slash(), true);
        assert_eq!("foo".with_trailing_slash(), false);
        assert_eq!("foo/".with_trailing_slash(), true);
        assert_eq!("foo/.".with_trailing_slash(), true);
        assert_eq!("foo/./".with_trailing_slash(), true);
        assert_eq!("foo/..".with_trailing_slash(), false);
    }

    #[test]
    fn test_try_add_trailing_slash() {
        let mut pathbuf = PathBuf::from("");
        pathbuf.try_add_trailing_slash();
        assert_eq!(pathbuf.as_os_str(), "");

        let mut pathbuf = PathBuf::from("/");
        pathbuf.try_add_trailing_slash();
        assert_eq!(pathbuf.as_os_str(), "/");

        let mut pathbuf = PathBuf::from("foo");
        pathbuf.try_add_trailing_slash();
        assert_eq!(pathbuf.as_os_str(), "foo/");

        let mut pathbuf = PathBuf::from("foo/");
        pathbuf.try_add_trailing_slash();
        assert_eq!(pathbuf.as_os_str(), "foo/");

        let mut pathbuf = PathBuf::from("foo/.");
        pathbuf.try_add_trailing_slash();
        assert_eq!(pathbuf.as_os_str(), "foo/.");

        let mut pathbuf = PathBuf::from("foo/./");
        pathbuf.try_add_trailing_slash();
        assert_eq!(pathbuf.as_os_str(), "foo/./");

        let mut pathbuf = PathBuf::from("foo/..");
        pathbuf.try_add_trailing_slash();
        assert_eq!(pathbuf.as_os_str(), "foo/../");
    }

    #[test]
    fn test_trailing_slash_problem_with_lstat() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let base_dir = "/tmp/test_lstat_path_with_trailing_slash";

                let file_name = String::from(base_dir) + "/" + "file";
                let dir_name = String::from(base_dir) + "/" + "dir";
                let link1_name = String::from(base_dir) + "/" + "link1";
                let link2_name = String::from(base_dir) + "/" + "link2";

                let result = std::panic::catch_unwind(|| {
                    // create a temporary dir for test
                    nc::mkdir(base_dir, 0o755).unwrap();

                    // init file and dir
                    File::create(&file_name).unwrap();
                    nc::mkdir(&dir_name, 0o755).unwrap();

                    nc::symlink(&file_name, &link1_name).unwrap();
                    nc::symlink(&dir_name, &link2_name).unwrap();

                    let mut stat = nc::stat_t::default();

                    // lstat("link1")
                    nc::lstat(&link1_name, &mut stat).unwrap();
                    assert_eq!((stat.st_mode as nc::mode_t & nc::S_IFMT), nc::S_IFLNK);

                    // lstat("link1/")
                    assert_eq!(
                        nc::lstat(format!("{}/", link1_name).as_str(), &mut stat),
                        Err(nc::ENOTDIR)
                    );

                    // lstat("link2")
                    nc::lstat(&link2_name, &mut stat).unwrap();
                    assert_eq!((stat.st_mode as nc::mode_t & nc::S_IFMT), nc::S_IFLNK);

                    // lstat("link2/")
                    nc::lstat(format!("{}/", link2_name).as_str(), &mut stat).unwrap();
                    assert_eq!((stat.st_mode as nc::mode_t & nc::S_IFMT), nc::S_IFDIR);
                });

                let _ = std::fs::remove_dir_all(base_dir);
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }

    #[test]
    fn test_trailing_slash_problem_with_mkdir() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let base_dir = "/tmp/test_trailing_slash_problem_with_mkdir";

                let dir1_name = String::from(base_dir) + "/" + "dir1";
                let dir2_name = String::from(base_dir) + "/" + "dir2";
                let dir3_name = String::from(base_dir) + "/" + "dir3";
                let link1_name = String::from(base_dir) + "/" + "link1";

                let result = std::panic::catch_unwind(|| {
                    // create a temporary dir for test
                    nc::mkdir(base_dir, 0o755).unwrap();

                    let mut stat = nc::stat_t::default();

                    // mkdir("dir1"), test mkdir without trailing slash
                    nc::mkdir(&dir1_name, 0o755).unwrap();
                    nc::lstat(&dir1_name, &mut stat).unwrap();
                    assert_eq!((stat.st_mode as nc::mode_t & nc::S_IFMT), nc::S_IFDIR);

                    // mkdir("dir2/"), test mkdir with trailing slash
                    nc::mkdir(format!("{}/", dir2_name).as_str(), 0o755).unwrap();
                    nc::lstat(format!("{}/", dir2_name).as_str(), &mut stat).unwrap();
                    assert_eq!((stat.st_mode as nc::mode_t & nc::S_IFMT), nc::S_IFDIR);

                    nc::symlink(&dir3_name, &link1_name).unwrap();

                    // mkdir("link1"), test mkdir with a symlink path without trailing slash
                    assert_eq!(nc::mkdir(&link1_name, 0o755), Err(nc::EEXIST));
                    assert_eq!(nc::lstat(&dir3_name, &mut stat), Err(nc::ENOENT));

                    // mkdir("link1/"), test mkdir with a symlink path with trailing slash
                    // Some sys-call(e.g. mkdir() and rmdir()) should never dereference the final
                    // component, even if the path contains a trailing slash.
                    assert_eq!(
                        nc::mkdir(format!("{}/", link1_name).as_str(), 0o755),
                        Err(nc::EEXIST)
                    );
                    assert_eq!(nc::lstat(&dir3_name, &mut stat), Err(nc::ENOENT));
                });

                let _ = std::fs::remove_dir_all(base_dir);
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }

    #[test]
    fn test_trailing_slash_problem_follow_symlink_recursively() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| {},
            || {
                let base_dir = "/tmp/test_trailing_slash_problem_follow_symlink_recursively";

                let dir_name = String::from(base_dir) + "/" + "dir";
                let link1_name = String::from(base_dir) + "/" + "link1";
                let link2_name = String::from(base_dir) + "/" + "link2";

                let result = std::panic::catch_unwind(|| {
                    // create a temporary dir for test
                    nc::mkdir(base_dir, 0o755).unwrap();

                    // init file and dir
                    nc::mkdir(&dir_name, 0o755).unwrap();

                    nc::symlink(&link2_name, &link1_name).unwrap();
                    nc::symlink(&dir_name, &link2_name).unwrap();

                    let mut stat = nc::stat_t::default();

                    // lstat("link1")
                    nc::lstat(&link1_name, &mut stat).unwrap();
                    assert_eq!((stat.st_mode as nc::mode_t & nc::S_IFMT), nc::S_IFLNK);

                    // lstat("link2")
                    nc::lstat(&link2_name, &mut stat).unwrap();
                    assert_eq!((stat.st_mode as nc::mode_t & nc::S_IFMT), nc::S_IFLNK);

                    // lstat("link1/") -> lstat("link2/") -> lstat("dir/")
                    nc::lstat(format!("{}/", link1_name).as_str(), &mut stat).unwrap();
                    assert_eq!((stat.st_mode as nc::mode_t & nc::S_IFMT), nc::S_IFDIR);
                });

                let _ = std::fs::remove_dir_all(base_dir);
                if let Err(err) = result {
                    std::panic::resume_unwind(err);
                }
            },
        )
    }
}
