use std::path::{Path, PathBuf, Component};
use errors::{Error, Result};
use filesystem::FileSystem;
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
            return Err(Error::invalid_argument(
                "when canonicalizing a relative path",
            ));
        }

        let mut it = user_path.components();
        // we need the `next` component to know if the current one is the last one
        let mut next_comp = it.next();

        while next_comp.is_some() {
            let component = next_comp.unwrap();
            next_comp = it.next();
            let is_last_component = next_comp.is_none();

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
                    guest_path.pop();
                    continue;
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

                    // For this latter case, we check that the symlink points to a directory once
                    // it is canonicalized, at the end of this loop.
                    if !is_last_component && !file_type.is_dir() && !file_type.is_symlink() {
                        return Err(Error::not_a_directory(
                            "when canonicalizing an intermediate path",
                        ));
                    }

                    // Nothing special to do if it's not a link or if we explicitly ask to not
                    // dereference 'user_path', as required by kernel like `lstat(2)`. Obviously,
                    // this later condition does not apply to intermediate path components.
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
    use nix::sys::stat::{S_IRWXU, S_IRWXG, S_IRWXO};
    use filesystem::FileSystem;
    use filesystem::binding::Binding;

    #[test]
    fn test_canonicalize_invalid_path() {
        // "/home/user" on the host, "/" on the guest
        let fs = FileSystem::with_root("/home/user");
        let path = PathBuf::from("/my/impossible/path");

        assert_eq!(
            fs.canonicalize(&path, false),
            Err(Error::no_such_file_or_dir("when substituting intermediary without glue"))
        );
    }

    #[test]
    fn test_canonicalize_normal_path() {
        // "/etc" on the host, "/" on the guest
        let mut fs = FileSystem::with_root("/etc");

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/acpi/./../acpi//events"), false)
                .unwrap(),
            PathBuf::from("/acpi/events")
        );

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/./../../.././../."), false)
                .unwrap(),
            PathBuf::from("/")
        );

        fs.set_root("/etc/acpi");
        fs.add_binding(Binding::new("/usr/bin", "/bin", true));

        // necessary, because nor "/bin" nor "/home" exist in "/etc/acpi"
        fs.set_glue_type(S_IRWXU | S_IRWXG | S_IRWXO);

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/bin/../home"), false)
                .unwrap(),
            PathBuf::from("/home")
        );
    }

    #[test]
    fn test_canonicalize_no_root_normal_path() {
        let mut fs = FileSystem::with_root("/");

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/home/../bin/./../bin/sleep"), false)
                .unwrap(),
            PathBuf::from("/bin/sleep")
        );

        // necessary, because nor "/test" probably doesn't exist
        fs.set_glue_type(S_IRWXU | S_IRWXG | S_IRWXO);

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/bin/../test"), false)
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
