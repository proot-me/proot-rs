use std::path::{Path, PathBuf, Component};
use std::io::Error as IOError;
use nix::Error;
use filesystem::fsnamespace::FileSystemNamespace;
use filesystem::substitution::Substitutor;

pub trait Canonicalizor {
    fn canonicalize(&self, path: &Path, deref_final: bool) -> Result<PathBuf, IOError>;
}

impl Canonicalizor for FileSystemNamespace {
    fn canonicalize(&self, user_path: &Path, deref_final: bool) -> Result<PathBuf, IOError> {
        let mut guest_path = PathBuf::new();

        if user_path.is_relative() {
            return Err(IOError::from(Error::invalid_argument()));
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
                    continue;
                }
                Component::ParentDir => {
                    if guest_path.pop() {
                        continue;
                    } else {
                        // the path is invalid, as it didn't manage to remove the last component
                        // (probably a path like "/..")
                        return Err(IOError::from(Error::invalid_argument()));
                    }
                }
                Component::Normal(path_part) => {
                    guest_path.push(path_part);
                    let (_, file_type) = self.substitute_intermediary_and_glue(&guest_path)?;

                    // Nothing special to do if it's not a link or if we
                    // explicitly ask to not dereference 'user_path', as
                    // required by syscalls like `lstat(2)`. Obviously, this
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
    use std::io::Error as IOError;
    use nix::Error;
    use nix::errno::Errno;
    use filesystem::fsnamespace::FileSystemNamespace;

    #[test]
    fn test_canonicalize_invalid_path() {
        let mut fs = FileSystemNamespace::new();

        // "/home/user" on the host, "/" on the guest
        fs.set_root("/home/user");

        let path = PathBuf::from("/../../../test");

        assert!(fs.canonicalize(&path, false).is_err());
    }

    #[test]
    fn test_canonicalize_impossible_path() {
        let mut fs = FileSystemNamespace::new();

        // "/home/user" on the host, "/" on the guest
        fs.set_root("/home/user");
        let path = PathBuf::from("/impossible/path/over/there");

        assert!(fs.canonicalize(&path, false).is_err());
    }

    #[test]
    fn test_canonicalize_normal_path() {
        let mut fs = FileSystemNamespace::new();

        // "/home/user" on the host, "/" on the guest
        fs.set_root("/etc");

        assert_eq!(
            fs.canonicalize(&PathBuf::from("/acpi/./../acpi//events"), false)
                .unwrap(),
            PathBuf::from("/acpi/events")
        );
    }
}
