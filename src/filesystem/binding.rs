use crate::errors::*;
use libc::PATH_MAX;
use nix::NixPath;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Host,  // in the real filesystem
    Guest, // in the sandbox
}

impl Side {
    pub fn reverse(&self) -> Side {
        match self {
            Side::Host => Side::Guest,
            Side::Guest => Side::Host,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Indicates a translation's direction.
///
/// For instance:
/// (Guest, Host) means the translation will move a path
/// from the `guest` filesystem (in the sandbox)
/// to the `host` filesystem (in the real filesystem).
pub struct Direction(pub Side, pub Side);

// TODO: Maybe we should canonicalize guest path during initialization
#[derive(Debug)]
pub struct Binding {
    host: PathBuf,
    guest: PathBuf,
    need_substitution: bool,
    _must_exist: bool,
}

impl Binding {
    //TODO: return Option<Binding> and make checks (test existence if must_exist,
    // sanitize, canon..)
    pub fn new<P1, P2>(host: P1, guest: P2, must_exist: bool) -> Binding
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        Binding {
            host: PathBuf::from(host.as_ref()),
            guest: PathBuf::from(guest.as_ref()),
            need_substitution: !host.as_ref().eq(guest.as_ref()),
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
    pub fn substitute_path_prefix(&self, path: &Path, from_side: Side) -> Result<PathBuf> {
        let current_prefix = self.get_path(from_side);

        // we start with the new prefix
        let mut new_path: PathBuf = PathBuf::from(self.get_path(from_side.reverse()));
        let stripped_path = path.strip_prefix(current_prefix).with_context(|| {
            format!(
                "Failed to strip_prefix {:?} from {:?}",
                current_prefix, path
            )
        })?;

        // and then add what remains of the path when removing the old prefix
        new_path.push(stripped_path);

        if new_path.len() >= PATH_MAX as usize {
            return Err(Error::errno_with_msg(
                Errno::ENAMETOOLONG,
                format!(
                    "Path length {} exceed PATH_MAX {}: {:?}",
                    new_path.len(),
                    PATH_MAX,
                    new_path
                ),
            ));
        }
        Ok(new_path)
    }
}

#[cfg(test)]
mod tests {
    use super::Side::{Guest, Host};
    use super::*;
    use crate::utils::tests::get_test_rootfs;
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
            binding.substitute_path_prefix(&PathBuf::from("/bin/sleep"), Guest),
            Ok(PathBuf::from("/home/user/bin/sleep"))
        ); // "/" => "/home/user"
        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/"), Guest),
            Ok(PathBuf::from("/home/user"))
        ); // same here
        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/bin/sleep"), Host),
            Err(Error::unknown())
        ); // "/home/user" is not a prefix of this path
        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/"), Host),
            Err(Error::unknown())
        ); // same here
    }

    #[test]
    fn test_substitute_path_prefix_different_path() {
        // "/etc" on host = "/media" on guest
        let binding = Binding::new("/etc", "/media", true);

        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/etc/bin/sleep"), Guest),
            Err(Error::unknown())
        ); // no "/etc" prefix on the guest side
        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/media/bin/sleep"), Guest,),
            Ok(PathBuf::from("/etc/bin/sleep"))
        ); // "/media" => "/etc"
        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/etc/bin/sleep"), Host),
            Ok(PathBuf::from("/media/bin/sleep"))
        ); // "/etc" => "/media"
        assert_eq!(
            binding.substitute_path_prefix(&PathBuf::from("/media/bin/sleep"), Host,),
            Err(Error::unknown())
        ); // no "/media" prefix on the host side
    }
}
