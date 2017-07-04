use std::path::{Path, PathBuf};
use std::cmp::Ordering;
use nix::Result;
use nix::errno::Errno;
use nix::Error;
use filesystem::bindings::{Binding, Direction, Side};
use filesystem::bindings::Side::{Host, Guest};

/// Information related to a file-system name-space.
#[derive(Debug)]
pub struct FileSystemNamespace {
    /// List of bindings used to replicate `mount` and `bind`.
    /// It will also contain the root binding (to replicate `chroot`).
    bindings: Vec<Binding>,
    /// Working directory, à la `/proc/self/pwd`.
    cwd: PathBuf,
    /// Host root (the binding associated to `/`)
    root: PathBuf
}

#[allow(dead_code)]

impl FileSystemNamespace {
    pub fn new() -> FileSystemNamespace {
        FileSystemNamespace {
            bindings: vec![],
            cwd: PathBuf::from("."),
            root: PathBuf::from("/")
        }
    }

    /// Add a binding at the beginning of the list,
    /// so that we get the most recent one when going through them
    /// in the `get_binding` method.
    pub fn add_binding(&mut self, binding: Binding) {
        self.bindings.insert(0, binding);
    }

    pub fn set_cwd(&mut self, cwd: &str) {
        self.cwd = PathBuf::from(cwd);
    }

    pub fn get_cwd(&self) -> &PathBuf {
        &self.cwd
    }

    pub fn set_root(&mut self, root: &str) {
        self.root = PathBuf::from(root);
        self.add_binding(Binding::new(root, "/", true));
    }

    /// Retrieves the first appropriate binding for a path translation.
    ///
    /// * `path` is the path which content will be tested on each binding
    /// * `side` indicates whether the starting side of the translation (guest for guest -> host)
    pub fn get_binding(&self, path: &Path, side: Side) -> Option<&Binding> {
        for binding in self.bindings.iter() {
            let binding_path = binding.get_path(side);

            if !path.starts_with(binding_path) {
                continue;
            }

            if side == Host && !self.root.eq(&PathBuf::from("/")) && self.belongs_to_guestfs(path) {
                // Avoid false positive when a prefix of the rootfs is
                // used as an asymmetric binding, ex.:
                //
                //     proot -m /usr:/location -r /usr/local/slackware
                //
                continue;
            }

            return Some(&binding);
        }

        None
    }

    /// Finds a suitable binding for the given path,
    /// and changes its prefix from one side to another, if it can.
    ///
    /// Returns returns the substituted path,
    /// or `None` if the path wasn't modified.
    ///
    /// * `path` is the path that will be modified. Must be canonicalized.
    /// * `direction` is the direction of the substitution.
    pub fn substitute_binding(&self, path: &Path, direction: Direction) -> Result<Option<PathBuf>> {
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

    pub fn translate_path(&self, user_path: &Path) -> Result<PathBuf> {
        let mut result = PathBuf::new();
        let user_path_is_absolute = user_path.is_absolute();

        if !user_path_is_absolute {
            // It is relative to the current working directory.
            result.push(self.get_cwd());

            //TODO: dir_fd != AT_FDCWD
        }

        //    status = notify_extensions(tracee, GUEST_PATH, (intptr_t) result, (intptr_t) user_path);
        //    if (status < 0)
        //        return status;
        //    if (status > 0)
        //        goto skip;

        result.push(user_path);
        //result = canonicalize_path(&result)?;

        let binding = self.get_binding(&result, Guest);

        //TODO: Finish

        Ok(result)
    }

    /// Translate a path from `guest` to `host`.
    /// Remove/substitute the leading part of a "translated" `path`.
    ///
    /// Returns
    /// * `Ok(None)` if no translation is required (ie. symmetric binding).
    /// * `Ok(PathBuf)` is the path was translated.
    /// * An error otherwise.
    pub fn detranslate_path(&self, path: &Path, referrer: Option<&Path>) -> Result<Option<PathBuf>> {
        // Don't try to detranslate relative paths (typically
        // the target of a relative symbolic link).
        if path.is_relative() {
            return Ok(None)
        }

        let mut follow_binding = true;

        // Is it a symlink?
        if let Some(referrer_path) = referrer {
            follow_binding = false;

            // In some cases bindings have to be resolved.
            if referrer_path.starts_with("/proc") {
                // Some links in "/proc" are generated dynamically by the kernel.
                // PRoot has to emulate some of them.
                //TODO: readlink_proc2
                unimplemented!(" /proc/.. referrer paths not supported!");
            } else if !self.belongs_to_guestfs(referrer_path) {
                let maybe_binding_referree = self.get_binding(path, Host);
                let binding_referrer = self.get_binding(referrer_path, Host).unwrap();

                // Resolve bindings for symlinks that belong
                // to a binding and point to the same binding.
                // For example, if "-b /lib:/foo" is specified
                // and the symlink "/lib/a -> /lib/b" exists
                // in the host rootfs namespace, then it
                // should appear as "/foo/a -> /foo/b" in the
                // guest rootfs namespace for consistency
                // reasons.
                if let Some(binding_referree) = maybe_binding_referree {
                    follow_binding =
                        binding_referree.get_path(Host) == binding_referrer.get_path(Host);
                }
            }
        }

        if follow_binding {
            if let Ok(maybe_path) = self.substitute_binding(path, Direction(Guest, Host)) {
                // if a suitable binding was found, we stop here
                return Ok(maybe_path);
            }
        }

        // otherwise, we simply try to strip the (guest) root
        if let Ok(stripped_path) = path.strip_prefix(&self.root) {
            Ok(Some(PathBuf::from("/").join(stripped_path)))
        } else {
            Ok(None)
        }
    }

    #[inline]
    /// Check if the translated @host_path belongs to the guest rootfs,
    /// that is, isn't from a binding.
    pub fn belongs_to_guestfs(&self, host_path: &Path) -> bool {
        host_path.starts_with(&self.root)
    }
}

/*
#[allow(dead_code)]
#[inline]
fn canonicalize_path(path: &PathBuf) -> Result<PathBuf> {
    match path.canonicalize() {
        Ok(canonicalized_path) => Ok(canonicalized_path),
        Err(err) => {
            match err.raw_os_error() {
                Some(errno) => Err(Error::Sys(Errno::from_i32(errno))),
                None => Err(Error::InvalidPath)
            }
        }
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use nix::Error;
    use nix::errno::Errno;
    use filesystem::bindings::{Binding, Side};
    use filesystem::bindings::Side::{Host, Guest};
    use filesystem::fsnamespace::FileSystemNamespace;

    #[test]
    fn test_belongs_to_guestfs() {
        let mut fs = FileSystemNamespace::new();

        fs.set_root("/etc");

        assert_eq!(fs.belongs_to_guestfs(Path::new("/etc")), true);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/etc/.")), true);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/etc/acpi/events")), true);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/acpi/events")), false);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/acpi")), false);
    }

    #[test]
    fn test_get_binding() {
        let mut fs = FileSystemNamespace::new();

        assert!(fs.get_binding(&PathBuf::from("/home/user"), Guest).is_none()); // no bindings
        assert!(fs.get_binding(&PathBuf::from("/home/user"), Host).is_none()); // no bindings

        // testing root binding
        fs.set_root("/home/user");

        assert_eq!(
            fs.get_binding(&Path::new("/bin"), Guest).unwrap().get_path(Guest),
            &PathBuf::from("/")); // it's "/home/user/bin" from the point of view of the host

        assert!(
            fs.get_binding(&Path::new("/etc"), Host)
            .is_none()); // "/etc" is outside of the guest fs, and no corresponding binding

        // testing binding outside of guest fs;
        // here, "/media" in the sandbox, is in reality "/etc" on the host
        fs.add_binding(Binding::new("/etc", "/media", true));

        assert_eq!(
            fs.get_binding(&Path::new("/media/folder/subfolder"), Guest).unwrap().get_path(Guest),
            &PathBuf::from("/media")); // it should detect the lastly-added binding

        assert_eq!(
            fs.get_binding(&Path::new("/etc/folder/subfolder"), Host).unwrap().get_path(Guest),
            &PathBuf::from("/media")); // same on the other side

        assert!(
            fs.get_binding(&Path::new("/bin"), Host)
            .is_none()); // should correspond to no binding

        // testing symmetric binding
        fs.add_binding(Binding::new("/bin", "/bin", true));

        assert_eq!(
            fs.get_binding(&Path::new("/bin/folder/subfolder"), Guest).unwrap().get_path(Guest),
            &PathBuf::from("/bin")); // it should detect the binding

        assert_eq!(
            fs.get_binding(&Path::new("/bin/folder/subfolder"), Host).unwrap().get_path(Guest),
            &PathBuf::from("/bin")); // it should detect the binding
    }

    #[test]
    fn test_substitute_binding() {
        let mut fs = FileSystemNamespace::new();

        fs.set_root("/home/user");

        // "/etc" on the host, "/media" on the guest
        fs.add_binding(Binding::new("/etc", "/media", true));

        assert_eq!(
            fs.substitute_binding(&Path::new("/etc/folder/subfolder"), Direction(Host, Guest)),
            Ok(Some(PathBuf::from("/media/folder/subfolder")))); // "/etc" => "/media"

        assert_eq!(
            fs.substitute_binding(&Path::new("/media/folder/subfolder"), Direction(Host, Guest)),
            Err(Error::Sys(Errno::ENOENT))); // the path isn't translatable to the guest fs

        assert_eq!(
            fs.substitute_binding(&Path::new("/etc/folder/subfolder"), Direction(Guest, Host)),
            Ok(Some(PathBuf::from("/home/user/etc/folder/subfolder")))); // "/" => "/home/user"

        assert_eq!(
            fs.substitute_binding(&Path::new("/media/folder/subfolder"), Direction(Guest, Host)),
            Ok(Some(PathBuf::from("/etc/folder/subfolder")))); // "/media" => "/etc"

        fs.add_binding(Binding::new("/etc/something", "/etc/something", true));

        assert_eq!(
            fs.substitute_binding(&Path::new("/etc/something/subfolder"), Direction(Guest, Host)),
            Ok(None) // the binding is symmetric, so no need to modify the path
        );

        assert_eq!(
            fs.substitute_binding(&Path::new("/etc/something/subfolder"), Direction(Host, Guest)),
            Ok(None) // same in the other direction
        );
    }

    #[test]
    fn test_detranslate_path_non_symlink() {
        let mut fs = FileSystemNamespace::new();

        // "/home/user" on the host, "/" on the guest
        fs.set_root("/home/user");

        assert_eq!(
            fs.detranslate_path(&Path::new("/bin/sleep"), None),
            Ok(Some(PathBuf::from("/home/user/bin/sleep"))) // "/" -> "/home/user"
        );

        assert_eq!(
            fs.detranslate_path(&Path::new("/"), None),
            Ok(Some(PathBuf::from("/home/user"))) // "/" -> "/home/user"
        );

        assert_eq!(
            fs.detranslate_path(&Path::new("/home/other_user"), None),
            Ok(Some(PathBuf::from("/home/user/home/other_user"))) // "/" -> "/home/user"
        );

        fs.add_binding(Binding::new("/etc/something", "/etc/something", true));

        assert_eq!(
            fs.detranslate_path(&Path::new("/etc/something/subfolder"), None),
            Ok(None) // because it's a symmetric binding
        );

        // "/etc/host" in the host, "/etc/guest" in the guest
        fs.add_binding(Binding::new("/etc/host", "/etc/guest", true));

        assert_eq!(
            fs.detranslate_path(&Path::new("/etc/guest/something"), None),
            Ok(Some(PathBuf::from("/etc/host/something")))); //

        //TODO: detranslate symlink tests
    }

    /*
    #[test]
    fn test_canonicalize_invalid_path() {
        let path = PathBuf::from("/../../../test");

        assert_eq!(canonicalize_path(&path), Err(Error::Sys(Errno::ENOENT)));
    }

    #[test]
    fn test_canonicalize_impossible_path() {
        let path = PathBuf::from("/impossible/path/over/there");

        assert_eq!(canonicalize_path(&path), Err(Error::Sys(Errno::ENOENT)));
    }

    #[test]
    fn test_canonicalize_normal_path() {
        let path = PathBuf::from("/home/../bin/./../bin/sleep");

        assert_eq!(canonicalize_path(&path), Ok(PathBuf::from("/bin/sleep")));
    }

    #[test]
    fn test_translate_path_absolute_normal_path_no_bindings() {
        let fs = FileSystemNamespace::new();
        let path = Path::new("/home/../bin/./../bin/sleep");

        assert_eq!(fs.translate_path(&path), Ok(PathBuf::from("/bin/sleep")));
    }

    #[test]
    fn test_translate_path_normal_absolute_path_with_root() {
        let mut fs = FileSystemNamespace::new();
        let path = Path::new("/acpi/events");

        fs.set_root("/etc");

        assert_eq!(fs.translate_path(&path), Ok(PathBuf::from("/etc/acpi/events")));
    }
    */
}