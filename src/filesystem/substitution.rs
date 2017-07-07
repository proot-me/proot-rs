use std::path::{Path, PathBuf};
use nix::Result;
use nix::errno::Errno;
use nix::Error;
use filesystem::binding::Direction;
use filesystem::fsnamespace::FileSystemNamespace;

pub trait Substitution {
    fn substitute_binding(&self, path: &Path, direction: Direction) -> Result<Option<PathBuf>>;
}

impl Substitution for FileSystemNamespace {
    /// Finds a suitable binding for the given path,
    /// and changes its prefix from one side to another, if it can.
    ///
    /// Returns the substituted path,
    /// or `None` if the path wasn't modified.
    ///
    /// * `path` is the path that will be modified. Must be canonicalized.
    /// * `direction` is the direction of the substitution.
    fn substitute_binding(&self, path: &Path, direction: Direction) -> Result<Option<PathBuf>> {
        let maybe_binding = self.get_binding(path, direction.0);

        if maybe_binding.is_none() {
            return Err(Error::Sys(Errno::ENOENT));
        }
        let binding = maybe_binding.unwrap();

        // Is it a "symmetric" binding?
        if !binding.needs_substitution() {
            return Ok(None);
        }

        Ok(binding.substitute_path_prefix(path, direction)?)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use nix::Error;
    use nix::errno::Errno;
    use filesystem::binding::Binding;
    use filesystem::binding::Side::{Host, Guest};
    use filesystem::fsnamespace::FileSystemNamespace;


    #[test]
    fn test_substitute_binding_root_and_asymmetric() {
        let mut fs = FileSystemNamespace::new();

        fs.set_root("/home/user");

        // "/etc" on the host, "/media" on the guest
        fs.add_binding(Binding::new("/etc", "/media", true));

        assert_eq!(
            fs.substitute_binding(&Path::new("/etc/folder/subfolder"), Direction(Host, Guest)),
            Ok(Some(PathBuf::from("/media/folder/subfolder")))
        ); // "/etc" => "/media"

        assert_eq!(
            fs.substitute_binding(
                &Path::new("/media/folder/subfolder"),
                Direction(Host, Guest),
            ),
            Err(Error::Sys(Errno::ENOENT))
        ); // the path isn't translatable to the guest fs

        assert_eq!(
            fs.substitute_binding(&Path::new("/etc/folder/subfolder"), Direction(Guest, Host)),
            Ok(Some(PathBuf::from("/home/user/etc/folder/subfolder")))
        ); // "/" => "/home/user"

        assert_eq!(
            fs.substitute_binding(
                &Path::new("/media/folder/subfolder"),
                Direction(Guest, Host),
            ),
            Ok(Some(PathBuf::from("/etc/folder/subfolder")))
        ); // "/media" => "/etc"
    }

    #[test]
    fn test_substitute_binding_symmetric() {
        let mut fs = FileSystemNamespace::new();

        fs.set_root("/home/user");
        fs.add_binding(Binding::new("/etc/something", "/etc/something", true));

        assert_eq!(
            fs.substitute_binding(
                &Path::new("/etc/something/subfolder"),
                Direction(Guest, Host),
            ),
            Ok(None) // the binding is symmetric, so no need to modify the path
        );

        assert_eq!(
            fs.substitute_binding(
                &Path::new("/etc/something/subfolder"),
                Direction(Host, Guest),
            ),
            Ok(None) // same in the other direction
        );
    }
}
