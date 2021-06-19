use crate::errors::*;
use crate::filesystem::substitution::Substitutor;
use crate::filesystem::FileSystem;
use std::path::{Component, Path, PathBuf};

use super::binding::Side;

pub trait Canonicalizer {
    fn canonicalize<P: AsRef<Path>>(&self, path: P, deref_final: bool) -> Result<PathBuf>;
}

impl Canonicalizer for FileSystem {
    /// Canonicalizes `guest_path` relative to the guest root (see `man 3
    /// realpath`).
    ///
    /// It removes ".." and "." from the paths and recursively dereferences
    /// symlinks. It checks that every path of the path exists.
    /// The result is a canonicalized path on the `Guest` side.
    ///
    /// The final component can be a path that does not exist. The final
    /// component is only deferenced if `deref_final` is true and path is
    /// existing.
    ///
    /// # Paramters
    ///
    /// - guest_path: path to be canonicalized, must be absolute path
    /// - deref_final: weather or not to dereference final user_path
    ///
    /// # Return
    ///
    /// guest_path_new: the canonicalized user_path, which is a path in the view
    /// of Guest
    ///
    /// # Note
    ///
    /// The current implementation performs a slight normalization on
    /// `guest_path` in advance, which is caused by `Path::components()`
    /// function. This means that `"/foo/bar/."` is equivalent to
    /// `"/foo/bar/"` and also to `"/foo/bar"`.
    ///
    /// This means that the final component of `"/foo/bar/."` is `"bar"`, and we
    /// will not check if `"/foo/bar/"` is a dir or not.
    ///
    /// # Error
    ///
    /// This function will return an error in the following situations:
    ///
    /// - The `guest_path` is a relative path.
    /// - An error occurred while calling `Substitutor::substitute()` to convert
    ///   to the host side path
    /// - A non-final component in path is not a directory.
    fn canonicalize<P: AsRef<Path>>(&self, guest_path: P, deref_final: bool) -> Result<PathBuf> {
        let guest_path = guest_path.as_ref();
        // The `guest_path` must be absolute path
        if guest_path.is_relative() {
            return Err(Error::errno_with_msg(
                Errno::EINVAL,
                format!("Cannot canonicalizing a relative path: {:?}", guest_path),
            ));
        }

        // build guest_path_new from user_path
        let mut guest_path_new = PathBuf::new();

        // split user_path to components and check them, so that path traversal can be
        // avoided.
        // We need the `next` component to know if the current one is the last one
        let mut it = guest_path.components();
        let mut next_comp = it.next();
        while let Some(component) = next_comp {
            next_comp = it.next();
            let is_last_component = next_comp.is_none();

            match component {
                Component::RootDir => {
                    guest_path_new.push(Component::RootDir);
                    continue;
                }
                Component::CurDir | Component::Prefix(_) => {
                    // Component::Prefix does not occur on Unix
                    continue;
                }
                Component::ParentDir => {
                    guest_path_new.pop();
                    continue;
                }
                Component::Normal(path_part) => {
                    guest_path_new.push(path_part);

                    // Resolve bindings and add glue if necessary
                    // TODO: replace with substitute_intermediary_and_glue() when glue is supported.
                    let host_path = self.substitute(&guest_path_new, Side::Guest)?;

                    let metadata = host_path.symlink_metadata();
                    // `metadata` is error if we cannot access this file or file is not exist.
                    // However we can accept this path because Some syscall (e.g. mkdir, mknod)
                    // allow final component not exist.
                    if is_last_component && metadata.is_err() {
                        continue;
                    }
                    // We can continue if we are now on the last component and are explicitly asked
                    // not to dereference 'user_path'.
                    if is_last_component && !deref_final {
                        continue;
                    }

                    let file_type = metadata?.file_type();

                    // directory can always push
                    if file_type.is_dir() {
                        continue;
                    }
                    if file_type.is_symlink() {
                        // we need to deref
                        // TODO: add test for this
                        let link_value = host_path.read_link()?;
                        let mut new_user_path = if link_value.is_absolute() {
                            // link_value is a absolute path, so we need to replace user_path
                            // with link_value first.
                            link_value
                        } else {
                            // link_value is a relative path, so we need to append link_value to
                            // guest_path_new.
                            guest_path_new.pop();
                            guest_path_new.push(&link_value);
                            guest_path_new
                        };
                        // append remaining Components
                        if let Some(comp) = next_comp {
                            new_user_path.push(comp);
                        }
                        it.for_each(|comp| new_user_path.push(comp));
                        // use new_user_path to call this function again and return
                        // TODO: Can be optimized by replacing `it`
                        return self.canonicalize(&new_user_path, deref_final);
                    }
                    // we cannot go through a path which is neither a directory nor a symlink
                    if !is_last_component {
                        return Err(Error::errno_with_msg(
                            Errno::ENOTDIR,
                            "when canonicalizing an intermediate path",
                        ));
                    }
                }
            }
        }

        Ok(guest_path_new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::FileSystem;
    use crate::utils::tests::get_test_rootfs_path;
    use nix::sys::stat::Mode;
    use std::path::PathBuf;

    #[test]
    fn test_canonicalize_invalid_path() {
        let fs = FileSystem::with_root(get_test_rootfs_path()).unwrap();

        // A path with non-existing final component is accepted.
        assert_eq!(
            fs.canonicalize("/non_existing_path", true),
            Ok("/non_existing_path".into())
        );
        assert_eq!(
            fs.canonicalize("/non_existing_path", false),
            Ok("/non_existing_path".into())
        );
        assert_eq!(
            fs.canonicalize("/etc/non_existing_path", true),
            Ok("/etc/non_existing_path".into())
        );
        assert_eq!(
            fs.canonicalize("/etc/non_existing_path", false),
            Ok("/etc/non_existing_path".into())
        );
        // Any non-final component in path should exist
        assert_eq!(
            fs.canonicalize("/etc/non_existing_path/non_existing_path", true),
            Err(Error::errno(Errno::ENOENT))
        );
        assert_eq!(
            fs.canonicalize("/etc/non_existing_path/non_existing_path", false),
            Err(Error::errno(Errno::ENOENT))
        );
        // Any non-final component in path should be directory
        assert_eq!(
            fs.canonicalize("/etc/passwd/non_existing_path", true),
            Err(Error::errno(Errno::ENOTDIR))
        );
        assert_eq!(
            fs.canonicalize("/etc/passwd/non_existing_path", false),
            Err(Error::errno(Errno::ENOTDIR))
        );
    }

    #[test]
    fn test_canonicalize_path_traversal() {
        let fs = FileSystem::with_root(get_test_rootfs_path()).unwrap();

        let path = PathBuf::from("/../non_existing_path");
        // should be ok, because ${rootfs}/non_existing_path exists on host
        assert_eq!(
            fs.canonicalize(&path, false),
            Ok("/non_existing_path".into())
        );
        // should be ok, because ${rootfs}/bin exists on host
        let path = PathBuf::from("/../bin");
        assert_eq!(fs.canonicalize(&path, false), Ok("/bin".into()));
    }
    #[test]
    fn test_canonicalize_normal_path() {
        let rootfs_path = get_test_rootfs_path();
        let fs = FileSystem::with_root(rootfs_path.as_path()).unwrap();

        assert_eq!(
            fs.canonicalize("/bin/./../bin//sleep", false).unwrap(),
            PathBuf::from("/bin/sleep")
        );

        assert_eq!(
            fs.canonicalize("/./../../.././../.", false).unwrap(),
            PathBuf::from("/")
        );
    }

    #[test]
    fn test_canonicalize_no_root_normal_path() {
        let mut fs = FileSystem::with_root(get_test_rootfs_path()).unwrap();

        // should be ok, because ${rootfs}/home, ${rootfs}/, ${rootfs}/bin/,
        // ${rootfs}/bin/sleep are all exist on host
        assert_eq!(
            fs.canonicalize(&PathBuf::from("/home/../etc/./../etc/passwd"), false)
                .unwrap(),
            PathBuf::from("/etc/passwd")
        );

        // necessary, because nor "/test" probably doesn't exist
        fs.set_glue_type(Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO);

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/etc/../test"), false)
                .unwrap(),
            PathBuf::from("/test")
        );
    }

    #[test]
    fn test_canonicalize_symlink_not_deref() {
        let fs = FileSystem::with_root(get_test_rootfs_path()).unwrap();

        // "${rootfs}/lib64" is a symlink to "lib"
        assert_eq!(
            fs.canonicalize(&PathBuf::from("/lib64"), false).unwrap(),
            PathBuf::from("/lib64")
        );
        assert_eq!(
            fs.canonicalize(&PathBuf::from("/lib64"), true).unwrap(),
            PathBuf::from("/lib")
        );
    }
}
