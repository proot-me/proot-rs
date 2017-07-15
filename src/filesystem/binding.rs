use std::path::{Path, PathBuf};
use libc::PATH_MAX;
use errors::{Error, Result};
use nix::NixPath;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Host, // in the real filesystem
    Guest, // in the sandbox
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Indicates a translation's direction.
///
/// For instance:
/// (Guest, Host) means the translation will move a path
/// from the `guest` filesystem (in the sandbox)
/// to the `host` filesystem (in the real filesystem).
pub struct Direction(pub Side, pub Side);

#[derive(Debug)]
pub struct Binding {
    host: PathBuf,
    guest: PathBuf,
    need_substitution: bool,
    _must_exist: bool,
}

impl Binding {
    //TODO: return Option<Binding> and make checks (test existence if must_exist, sanitize, canon..)
    pub fn new(host: &str, guest: &str, must_exist: bool) -> Binding {
        Binding {
            host: PathBuf::from(host),
            guest: PathBuf::from(guest),
            need_substitution: !host.eq(guest),
            _must_exist: must_exist,
        }
    }

    #[inline]
    pub fn get_path(&self, side: Side) -> &PathBuf {
        match side {
            Side::Guest => &self.guest,
            Side::Host => &self.host,
        }
    }

    #[inline]
    pub fn needs_substitution(&self) -> bool {
        self.need_substitution
    }


    #[inline]
    pub fn substitute_path_prefix(
        &self,
        path: &Path,
        direction: Direction,
    ) -> Result<Option<PathBuf>> {
        if direction.0 == direction.1 {
            return Ok(None);
        }

        let current_prefix = self.get_path(direction.0);

        // we start with the new prefix
        let mut new_path: PathBuf = PathBuf::from(self.get_path(direction.1));
        let stripped_path = path.strip_prefix(current_prefix);

        if stripped_path.is_err() {
            return Ok(None);
        }

        // and then add what remains of the path when removing the old prefix
        new_path.push(stripped_path.unwrap());

        if new_path.len() >= PATH_MAX as usize {
            return Err(Error::name_too_long());
        }

        Ok(Some(new_path))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::Side::{Host, Guest};
    use std::path::PathBuf;

    #[test]
    fn test_binding_get_path() {
        // "/etc" on host = "/media" on guest
        let binding = Binding::new("/etc", "/media", true);

        assert_eq!(binding.get_path(Side::Host), &PathBuf::from("/etc"));
        assert_eq!(binding.get_path(Side::Guest), &PathBuf::from("/media"));
    }

    #[test]
    fn test_substitute_path_prefix_root() {
        // "/etc" on host = "/media" on guest
        let binding = Binding::new("/home/user", "/", true);

        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/bin/sleep"), Direction(Guest, Host)),
            Ok(Some(PathBuf::from("/home/user/bin/sleep")))
        ); // "/" => "/home/user"
        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/"), Direction(Guest, Host)),
            Ok(Some(PathBuf::from("/home/user")))
        ); // same here
        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/bin/sleep"), Direction(Host, Guest)),
            Ok(None)
        ); // "/home/user" is not a prefix of this path
        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/"), Direction(Host, Guest)),
            Ok(None)
        ); // same here
    }

    #[test]
    fn test_substitute_path_prefix_different_path() {
        // "/etc" on host = "/media" on guest
        let binding = Binding::new("/etc", "/media", true);

        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/etc/bin/sleep"), Direction(Guest, Host)),
            Ok(None)
        ); // no "/etc" prefix on the guest side
        assert_eq!(
            binding.substitute_path_prefix(
                &PathBuf::from("/media/bin/sleep"),
                Direction(Guest, Host),
            ),
            Ok(Some(PathBuf::from("/etc/bin/sleep")))
        ); // "/media" => "/etc"
        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/etc/bin/sleep"), Direction(Host, Guest)),
            Ok(Some(PathBuf::from("/media/bin/sleep")))
        ); // "/etc" => "/media"
        assert_eq!(
            binding.substitute_path_prefix(
                &PathBuf::from("/media/bin/sleep"),
                Direction(Host, Guest),
            ),
            Ok(None)
        ); // no "/media" prefix on the host side
    }
}
