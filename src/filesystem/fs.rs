use std::ffi::CString;
use std::fs::Metadata;
use std::path::{Path, PathBuf};

use nix::sys::{self, stat::Mode};
use nix::unistd::{self, AccessFlags};

use crate::errors::{Error, Result, WithContext};
use crate::filesystem::binding::Side::Host;
use crate::filesystem::binding::{Binding, Side};

/// Information related to a file-system name-space.
#[derive(Debug)]
pub struct FileSystem {
    /// List of bindings used to replicate `mount` and `bind`.
    /// It will also contain the root binding (to replicate `chroot`).
    bindings: Vec<Binding>,
    /// Working directory in gusetfs, à la `/proc/self/pwd`.
    cwd: PathBuf,
    /// Guest root (the binding associated to `/`)
    root: PathBuf,
    /// Use for glue (//TODO: explain when implemented)
    glue_type: Mode,
}

impl FileSystem {
    pub fn new() -> FileSystem {
        FileSystem {
            bindings: vec![],
            cwd: PathBuf::from("."),
            root: PathBuf::from("/"),
            glue_type: Mode::empty(),
        }
    }

    #[cfg(test)]
    pub fn with_root<P: AsRef<Path>>(root: P) -> FileSystem {
        let mut file_system = FileSystem::new();

        file_system.set_root(root);
        file_system
    }

    /// Add a binding at the beginning of the list,
    /// so that we get the most recent one when going through them
    /// in the `get_binding` method.
    //TODO: sort bindings to make substitution of nested bindings deterministic
    #[inline]
    pub fn add_binding(&mut self, binding: Binding) {
        self.bindings.insert(0, binding);
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
    /// * `from_side` indicates the starting side of the translation (ie. guest
    ///   for guest -> host)
    pub fn get_first_appropriate_binding(&self, path: &Path, from_side: Side) -> Option<&Binding> {
        for binding in self.bindings.iter() {
            let binding_path = binding.get_path(from_side);

            if !path.starts_with(binding_path) {
                continue;
            }

            // TODO: Do we really need to find binding from host to guest?
            if from_side == Host
                && !self.root.eq(&PathBuf::from("/"))
                && self.belongs_to_guestfs(path)
            {
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

    #[inline]
    /// Checks is `path` is a file, does exist and is executable.
    pub fn is_path_executable(&self, path: &Path) -> Result<()> {
        unistd::access(path, AccessFlags::F_OK)?;
        unistd::access(path, AccessFlags::X_OK)?;
        sys::stat::lstat(path)?;
        Ok(())
    }

    #[inline]
    pub fn set_cwd(&mut self, cwd: PathBuf) {
        self.cwd = cwd;
    }

    #[inline]
    pub fn get_cwd(&self) -> &Path {
        &self.cwd
    }

    #[inline]
    pub fn set_root<P: AsRef<Path>>(&mut self, root: P) {
        self.root = root.as_ref().into();
        self.add_binding(Binding::new(root, "/", true));
    }

    #[inline]
    pub fn get_root(&self) -> &Path {
        &self.root
    }

    #[inline]
    pub fn get_glue_type(&self) -> &Mode {
        &self.glue_type
    }

    #[inline]
    pub fn set_glue_type(&mut self, mode: Mode) {
        self.glue_type = mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::binding::Binding;
    use crate::filesystem::binding::Side::{Guest, Host};
    use crate::utils::tests::get_test_rootfs;
    use std::path::{Path, PathBuf};

    // TODO: consider remove this test
    #[test]
    fn test_fs_belongs_to_guestfs() {
        // this test does not trigger real file access, so we do not call
        // `get_test_rootfs()` here.
        let fs = FileSystem::with_root("/etc");

        assert_eq!(fs.belongs_to_guestfs(Path::new("/etc")), true);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/etc/.")), true);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/etc/acpi/events")), true);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/acpi/events")), false);
        assert_eq!(fs.belongs_to_guestfs(Path::new("/acpi")), false);
    }

    #[test]
    fn test_fs_get_binding() {
        // this test does not trigger real file access, so we do not call
        // `get_test_rootfs()` here.

        let mut fs = FileSystem::new();

        assert!(fs
            .get_first_appropriate_binding(&PathBuf::from("/home/user"), Guest)
            .is_none()); // no bindings
        assert!(fs
            .get_first_appropriate_binding(&PathBuf::from("/home/user"), Host)
            .is_none()); // no bindings

        // testing root binding
        fs.set_root("/home/user");

        assert_eq!(
            fs.get_first_appropriate_binding(&Path::new("/bin"), Guest)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/")
        ); // it's "/home/user/bin" from the point of view of the host

        assert!(fs
            .get_first_appropriate_binding(&Path::new("/etc"), Host)
            .is_none()); // "/etc" is outside of the guest fs, so no corresponding binding found

        // testing binding outside of guest fs;
        // here, "/etc" on the host corresponds to "/media" in the sandbox.
        fs.add_binding(Binding::new("/etc", "/media", true));

        assert_eq!(
            fs.get_first_appropriate_binding(&Path::new("/media/folder/subfolder"), Guest)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/media")
        ); // it should detect the lastly-added binding

        assert_eq!(
            fs.get_first_appropriate_binding(&Path::new("/etc/folder/subfolder"), Host)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/media")
        ); // same on the other side

        assert!(fs
            .get_first_appropriate_binding(&Path::new("/bin"), Host)
            .is_none()); // should correspond to no binding

        // testing symmetric binding
        fs.add_binding(Binding::new("/bin", "/bin", true));

        assert_eq!(
            fs.get_first_appropriate_binding(&Path::new("/bin/folder/subfolder"), Guest)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/bin")
        ); // it should detect the binding

        assert_eq!(
            fs.get_first_appropriate_binding(&Path::new("/bin/folder/subfolder"), Host)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/bin")
        ); // same on the other side
    }

    #[test]
    fn test_fs_is_path_executable() {
        let fs = FileSystem::with_root(get_test_rootfs());

        assert!(fs.is_path_executable(&PathBuf::from("/bin/sleep")).is_ok());
        assert!(fs.is_path_executable(&PathBuf::from("/../sleep")).is_err());
    }
}
