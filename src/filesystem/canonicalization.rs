use std::path::{Path, PathBuf, Component};
use nix::{Result, Error};
use nix::errno::Errno;
use filesystem::fs::FileSystem;
use filesystem::substitution::Substitutor;

pub trait Canonicalizer {
    fn canonicalize(&self, path: &Path, deref_final: bool) -> Result<PathBuf>;
}

impl Canonicalizer for FileSystem {
    /// Canonicalizes `user_path` relative to the guest root (see `man 3 realpath`).
    ///
    /// It removes ".." and "." from the paths and recursively dereferences symlinks.
    /// It checks that every path of the path exists.
    /// The result is a canonicalized path on the `Guest` side.
    ///
    /// The final path is only deferenced if `deref_final` is true.
    fn canonicalize(&self, user_path: &Path, deref_final: bool) -> Result<PathBuf> {
        let mut guest_path = PathBuf::new();

        if user_path.is_relative() {
            return Err(Error::invalid_argument());
        }

        let mut it = user_path.components();
        // we need the `next` component to know if the current one is the last one
        let mut maybe_next_component = it.next();

        while maybe_next_component.is_some() {
            let component = maybe_next_component.unwrap();
            maybe_next_component = it.next();
            let is_last_component = maybe_next_component.is_none();

            match component {
                Component::RootDir => {
                    guest_path.push("/");
                    continue;
                }
                Component::CurDir |
                Component::Prefix(_) => {
                    // Component::Prefix does not occur on Unix
                    continue;
                }
                Component::ParentDir => {
                    if guest_path.pop() {
                        continue;
                    } else {
                        // the path is invalid, as it didn't manage to remove the last component
                        // (it's probably a path like "/..").
                        return Err(Error::invalid_argument());
                    }
                }
                Component::Normal(path_part) => {
                    guest_path.push(path_part);

                    // Resolve bindings and add glue if necessary
                    let (_, maybe_file_type) = self.substitute_intermediary_and_glue(&guest_path)?;

                    //TODO: remove when glue is implemented
                    if maybe_file_type.is_none() {
                        continue;
                    }
                    let file_type = maybe_file_type.unwrap();

                    // Checks that a non-final component exists and
                    // either is a directory or is a symlink.
                    // For this latter case, we check that the
                    // symlink points to a directory once it is
                    // canonicalized, at the end of this loop.
                    if !is_last_component && !file_type.is_dir() && !file_type.is_symlink() {
                        return Err(Error::Sys(Errno::ENOTDIR));
                    }

                    // Nothing special to do if it's not a link or if we
                    // explicitly ask to not dereference 'user_path', as
                    // required by kernel like `lstat(2)`. Obviously, this
                    // later condition does not apply to intermediate path
                    // components.
                    if file_type.is_dir() || (is_last_component && !deref_final) {
                        continue;
                    } else {
                        //TODO: deref symlink
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
    use std::path::PathBuf;
    use filesystem::fs::FileSystem;
    use filesystem::binding::Binding;

    #[test]
    fn test_canonicalize_invalid_path() {
        let mut fs = FileSystem::new();

        // "/home/user" on the host, "/" on the guest
        fs.set_root("/home/user");

        let path = PathBuf::from("/../../../test");

        assert!(fs.canonicalize(&path, false).is_err());
    }

    #[test]
    fn test_canonicalize_normal_path() {
        let mut fs = FileSystem::new();

        // "/etc" on the host, "/" on the guest
        fs.set_root("/etc");

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/acpi/./../acpi//events"), false)
                .unwrap(),
            PathBuf::from("/acpi/events")
        );

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/acpi/./../acpi//events"), false)
                .unwrap(),
            PathBuf::from("/acpi/events")
        );

        fs.set_root("/etc/acpi");
        fs.add_binding(Binding::new("/usr/bin", "/bin", true));

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/bin/../home"), false)
                .unwrap(),
            PathBuf::from("/home")
        );
    }

    #[test]
    fn test_canonicalize_no_root_normal_path() {
        let mut fs = FileSystem::new();

        // "/etc" on the host, "/" on the guest
        fs.set_root("/");

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/home/../bin/./../bin/sleep"), false)
                .unwrap(),
            PathBuf::from("/bin/sleep")
        );

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/bin/../test"), false)
                .unwrap(),
            PathBuf::from("/test")
        );
    }

    #[test]
    fn test_canonicalize_symlink_not_deref() {
        let mut fs = FileSystem::new();

        // "/etc" on the host, "/" on the guest
        fs.set_root("/bin");

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/sh"), false).unwrap(),
            PathBuf::from("/sh")
        );
    }
}
