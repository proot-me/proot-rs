use crate::errors::*;
use crate::filesystem::substitution::Substitutor;
use crate::filesystem::FileSystem;
use std::path::{Component, Path, PathBuf};

pub trait Canonicalizer {
    fn canonicalize(&self, path: &Path, deref_final: bool) -> Result<PathBuf>;
}

impl Canonicalizer for FileSystem {
    /// Canonicalizes `user_path` relative to the guest root (see `man 3
    /// realpath`).
    ///
    /// It removes ".." and "." from the paths and recursively dereferences
    /// symlinks. It checks that every path of the path exists.
    /// The result is a canonicalized path on the `Guest` side.
    ///
    /// The final path is only deferenced if `deref_final` is true.
    ///
    /// # Paramters
    ///
    /// - user_path: path to be canonicalized, must be absolute path
    /// - deref_final: weather or not to dereference final user_path
    ///
    /// # Return
    ///
    /// guest_path: the canonicalized user_path, which is a path in the view of
    /// Guest
    fn canonicalize(&self, user_path: &Path, deref_final: bool) -> Result<PathBuf> {
        // The `user_path` must be absolute path
        if user_path.is_relative() {
            return Err(Error::errno_with_msg(
                Errno::EINVAL,
                format!("Cannot canonicalizing a relative path: {:?}", user_path),
            ));
        }

        // build guest_path from user_path
        let mut guest_path = PathBuf::new();

        // split user_path to components and check them, so that path traversal can be
        // avoided.
        // We need the `next` component to know if the current one is the last one
        let mut it = user_path.components();
        let mut next_comp = it.next();
        while let Some(component) = next_comp {
            next_comp = it.next();
            let is_last_component = next_comp.is_none();

            match component {
                Component::RootDir => {
                    guest_path.push(Component::RootDir);
                    continue;
                }
                Component::CurDir | Component::Prefix(_) => {
                    // Component::Prefix does not occur on Unix
                    continue;
                }
                Component::ParentDir => {
                    guest_path.pop();
                    continue;
                }
                Component::Normal(path_part) => {
                    guest_path.push(path_part);

                    // Resolve bindings and add glue if necessary
                    // TODO: currently we check and ensure that all the path exist on host, but
                    // some syscall (e.g. mkdir, mknod) allow path not exist.
                    let (host_path, maybe_file_type) =
                        self.substitute_intermediary_and_glue(&guest_path)?;

                    //TODO: remove when glue is implemented
                    if maybe_file_type.is_none() {
                        continue;
                    }
                    let file_type = maybe_file_type.unwrap();

                    // directory can always push
                    if file_type.is_dir() {
                        continue;
                    }
                    if file_type.is_symlink() {
                        // we can continue if current path is symlink and is last component and
                        // if we explicitly ask to not dereference 'user_path', as required by
                        // kernel like `lstat(2)`
                        if is_last_component && !deref_final {
                            continue;
                        }
                        // we need to deref
                        // TODO: add test for this
                        let link_value = host_path.read_link()?;
                        let mut new_user_path = if link_value.is_absolute() {
                            // link_value is a absolute path, so we need to replace user_path
                            // with link_value first.
                            link_value
                        } else {
                            // link_value is a relative path, so we need to append link_value to
                            // guest_path.
                            guest_path.pop();
                            guest_path.push(&link_value);
                            guest_path
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

        Ok(guest_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::binding::Binding;
    use crate::filesystem::FileSystem;
    use nix::sys::stat::Mode;
    use std::path::PathBuf;

    #[test]
    fn test_canonicalize_invalid_path() {
        // "/home/user" on the host, "/" on the guest
        let fs = FileSystem::with_root("/home/user");
        let path = PathBuf::from("/my/impossible/path");

        assert_eq!(
            fs.canonicalize(&path, false),
            Err(Error::errno(Errno::ENOENT))
        );
    }

    #[test]
    fn test_canonicalize_path_traversal() {
        let fs = FileSystem::with_root("/usr");

        let path = PathBuf::from("/../usr/bin");
        // should be failed, because no /usr/usr/bin on host
        assert_eq!(
            fs.canonicalize(&path, false),
            Err(Error::errno(Errno::ENOENT))
        );
        // should be ok, because /usr/bin exist on host
        let path = PathBuf::from("/../bin");
        assert_eq!(fs.canonicalize(&path, false), Ok(PathBuf::from("/bin")));
    }
    #[test]
    fn test_canonicalize_normal_path() {
        // "/etc" on the host, "/" on the guest
        let mut fs = FileSystem::with_root("/usr");

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/bin/./../bin//sleep"), false)
                .unwrap(),
            PathBuf::from("/bin/sleep")
        );

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/./../../.././../."), false)
                .unwrap(),
            PathBuf::from("/")
        );

        fs.set_root("/etc");
        fs.add_binding(Binding::new("/usr/bin", "/bin", true));

        // necessary, because nor "/bin" nor "/home" exist in "/etc"
        fs.set_glue_type(Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO);

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/bin/../home"), false)
                .unwrap(),
            PathBuf::from("/home")
        );
    }

    #[test]
    fn test_canonicalize_no_root_normal_path() {
        let mut fs = FileSystem::with_root("/");

        // should be ok, because /home, /, /bin/, /bin/sleep are all exist on host
        assert_eq!(
            fs.canonicalize(&PathBuf::from("/home/../etc/./../etc/hostname"), false)
                .unwrap(),
            PathBuf::from("/etc/hostname")
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
        // "/bin" on the host, "/" on the guest
        let fs = FileSystem::with_root("/bin");

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/sh"), false).unwrap(),
            PathBuf::from("/sh")
        ); // "/sh" is a symlink, and is not dereferenced
    }
}
