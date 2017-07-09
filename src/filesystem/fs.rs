use std::path::{Path, PathBuf};
use std::fs::Metadata;
use nix::{Error, Result};
use nix::errno::Errno;
use filesystem::binding::{Binding, Side};
use filesystem::binding::Side::Host;

/// Information related to a file-system name-space.
#[derive(Debug)]
pub struct FileSystem {
    /// List of bindings used to replicate `mount` and `bind`.
    /// It will also contain the root binding (to replicate `chroot`).
    bindings: Vec<Binding>,
    /// Working directory, à la `/proc/self/pwd`.
    cwd: PathBuf,
    /// Guest root (the binding associated to `/`)
    root: PathBuf,
}

#[allow(dead_code)]

impl FileSystem {
    pub fn new() -> FileSystem {
        FileSystem {
            bindings: vec![],
            cwd: PathBuf::from("."),
            root: PathBuf::from("/"),
        }
    }

    /// Add a binding at the beginning of the list,
    /// so that we get the most recent one when going through them
    /// in the `get_binding` method.
    ///
    //TODO: sort bindings to make substitution of nested bindings deterministic
    #[inline]
    pub fn add_binding(&mut self, binding: Binding) {
        self.bindings.insert(0, binding);
    }

    #[inline]
    pub fn set_cwd(&mut self, cwd: &str) {
        self.cwd = PathBuf::from(cwd);
    }

    #[inline]
    pub fn get_cwd(&self) -> &PathBuf {
        &self.cwd
    }

    #[inline]
    pub fn set_root(&mut self, root: &str) {
        self.root = PathBuf::from(root);
        self.add_binding(Binding::new(root, "/", true));
    }

    #[inline]
    pub fn get_root(&self) -> &PathBuf {
        &self.root
    }

    #[inline]
    /// Checks if the translated `host_path` belongs to the guest rootfs,
    /// that is, if it isn't from a binding.
    pub fn belongs_to_guestfs(&self, host_path: &Path) -> bool {
        host_path.starts_with(&self.root)
    }

    /// Retrieves the first appropriate binding for a path translation.
    ///
    /// * `path` is the path which content will be tested on each binding
    /// * `side` indicates the starting side of the translation (ie. guest for guest -> host)
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

    //TODO: use cache
    #[inline]
    /// Retrieves the path's metadata without going through symlinks.
    pub fn get_direct_metadata(&self, path: &Path) -> Result<Metadata> {
        //        /* Don't notify extensions during the initialization of a binding.  */
        //        if (tracee->glue_type == 0) {
        //            status = notify_extensions(tracee, HOST_PATH, (intptr_t)host_path, finality);
        //            if (status < 0)
        //            return status;
        //        }

        // indirect call to `lstat`

        match path.symlink_metadata() {
            Ok(metadata) => Ok(metadata),
            Err(err) => {
                match err.raw_os_error() {
                    Some(errno) => Err(Error::Sys(Errno::from_i32(errno))),
                    None => Err(Error::InvalidPath),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use filesystem::binding::Binding;
    use filesystem::binding::Side::{Host, Guest};

    #[test]
    fn test_belongs_to_guestfs() {
        let mut fs = FileSystem::new();

        fs.set_root("/etc");

        assert_eq!(fs.belongs_to_guestfs(Path::new("/etc")), true);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/etc/.")), true);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/etc/acpi/events")), true);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/acpi/events")), false);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/acpi")), false);
    }

    #[test]
    fn test_get_binding() {
        let mut fs = FileSystem::new();

        assert!(
            fs.get_binding(&PathBuf::from("/home/user"), Guest)
                .is_none()
        ); // no bindings
        assert!(fs.get_binding(&PathBuf::from("/home/user"), Host).is_none()); // no bindings

        // testing root binding
        fs.set_root("/home/user");

        assert_eq!(
            fs.get_binding(&Path::new("/bin"), Guest)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/")
        ); // it's "/home/user/bin" from the point of view of the host

        assert!(fs.get_binding(&Path::new("/etc"), Host).is_none()); // "/etc" is outside of the guest fs, so no corresponding binding found

        // testing binding outside of guest fs;
        // here, "/etc" on the host corresponds to "/media" in the sandbox.
        fs.add_binding(Binding::new("/etc", "/media", true));

        assert_eq!(
            fs.get_binding(&Path::new("/media/folder/subfolder"), Guest)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/media")
        ); // it should detect the lastly-added binding

        assert_eq!(
            fs.get_binding(&Path::new("/etc/folder/subfolder"), Host)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/media")
        ); // same on the other side

        assert!(fs.get_binding(&Path::new("/bin"), Host).is_none()); // should correspond to no binding

        // testing symmetric binding
        fs.add_binding(Binding::new("/bin", "/bin", true));

        assert_eq!(
            fs.get_binding(&Path::new("/bin/folder/subfolder"), Guest)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/bin")
        ); // it should detect the binding

        assert_eq!(
            fs.get_binding(&Path::new("/bin/folder/subfolder"), Host)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/bin")
        ); // same on the other side
    }
}
