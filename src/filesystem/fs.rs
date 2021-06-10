use std::path::{Path, PathBuf};

use nix::sys::{self, stat::Mode};
use nix::unistd::{self, AccessFlags};

use crate::errors::*;
use crate::filesystem::binding::Side::Host;
use crate::filesystem::binding::{Binding, Side};

use super::{Canonicalizer, Substitutor};

/// The file-system information associated with one or more tracee, which
/// corresponds to the [`fs_struct`] structure in the kernel. If clone() is
/// called with `CLONE_FS` set, then both tracee will share this structure,
/// otherwise a copy will be created.
///
/// [`fs_struct`]: https://elixir.bootlin.com/linux/latest/source/include/linux/fs_struct.h
#[derive(Debug)]
pub struct FileSystem {
    /// List of bindings used to replicate `mount` and `bind`.
    /// It will also contain the root binding (to replicate `chroot`).
    bindings: Vec<Binding>,
    /// Working directory in guestfs, e.g., `/proc/self/cwd`, is always absolute
    /// and canonical path.
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
            cwd: PathBuf::from("/"),
            root: PathBuf::from("/"),
            glue_type: Mode::empty(),
        }
    }

    #[cfg(test)]
    pub fn with_root<P: AsRef<Path>>(root: P) -> Result<FileSystem> {
        let mut file_system = FileSystem::new();

        file_system.set_root(root)?;
        Ok(file_system)
    }

    /// Add a `host_path` to `guest_path` binding.
    /// `guest_path` must exist and be an absolute path.
    //TODO: sort bindings to make substitution of nested bindings deterministic
    #[inline]
    pub fn add_binding<P1, P2>(&mut self, host_path: P1, guest_path: P2) -> Result<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let canonical_host_path = std::fs::canonicalize(host_path)?;
        // TODO: allow path not existed when glue is implemented
        let canonical_guest_path = self.canonicalize(guest_path.as_ref(), true)?;
        // Add a binding at the beginning of the list, so that we get the most recent
        // one when going through them in the `get_binding` method.
        self.bindings.insert(
            0,
            Binding::new(canonical_host_path, canonical_guest_path, true),
        );
        Ok(())
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
    /// Checks if a `host_path` is a file, does exist and is executable.
    pub fn check_host_path_executable(host_path: &Path) -> Result<()> {
        // FIXME: It should not be used in syscal translate, because the permissions of
        // the tracee process should be checked
        unistd::access(host_path, AccessFlags::F_OK)?;
        unistd::access(host_path, AccessFlags::X_OK)?;
        sys::stat::lstat(host_path)?;
        Ok(())
    }

    /// Get current work directory (cwd)
    /// This function will return a guest side path, which is always canonical.
    pub fn get_cwd(&self) -> &Path {
        let cwd = &self.cwd;
        if cwd.is_relative() {
            warn!(
                "cwd of tracee is not absolute, there may be some bugs: {:?}",
                cwd
            );
        }
        cwd
    }

    /// Set current work directory (cwd) for this FileSystem instance.
    /// `guest_path` should be an absolute path and will be checked for access
    /// permissions.
    pub fn set_cwd<P: AsRef<Path>>(&mut self, guest_path: P) -> Result<()> {
        let guest_path = guest_path.as_ref();
        if guest_path.is_relative() {
            return Err(Error::errno_with_msg(
                Errno::EINVAL,
                format!(
                    "current work directory should be a relative path: {:?}",
                    guest_path
                ),
            ));
        }

        let guest_path_canonical = self.canonicalize(&guest_path, true)?;
        let host_path = self.substitute(&guest_path_canonical, Side::Guest)?;

        // To change cwd to a dir, the tracee must have execute (`x`) permission to this
        // dir, FIXME: This may be wrong, because we need to check if tracee has
        // permission
        nix::unistd::access(&host_path, AccessFlags::X_OK)?;

        self.cwd = guest_path_canonical;
        Ok(())
    }

    /// Set root directory for this FileSystem instance.
    /// The root path needs to be a host side path, and relative path are also
    /// accepted.
    #[inline]
    pub fn set_root<P: AsRef<Path>>(&mut self, host_path: P) -> Result<()> {
        let raw_root = host_path.as_ref();
        // the `root` is host path, we use host side canonicalize() to canonicalize it
        let canonical_root = std::fs::canonicalize(raw_root)?;
        self.root = canonical_root.clone();
        self.add_binding(canonical_root, "/")?;
        Ok(())
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
    use crate::filesystem::binding::Side::{Guest, Host};
    use crate::utils;
    use crate::utils::tests::get_test_rootfs_path;
    use std::path::{Path, PathBuf};

    // TODO: consider remove this test
    #[test]
    fn test_fs_belongs_to_guestfs() {
        // this test does not trigger real file access, so we do not call
        // `get_test_rootfs()` here.
        let fs = FileSystem::with_root("/etc").unwrap();

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
        fs.set_root(get_test_rootfs_path()).unwrap();

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
        // here, "/etc" on the host corresponds to "/tmp" in the sandbox.
        fs.add_binding("/etc", "/tmp").unwrap();

        assert_eq!(
            fs.get_first_appropriate_binding(&Path::new("/tmp/folder/subfolder"), Guest)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/tmp")
        ); // it should detect the lastly-added binding

        assert_eq!(
            fs.get_first_appropriate_binding(&Path::new("/etc/folder/subfolder"), Host)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/tmp")
        ); // same on the other side

        assert!(fs
            .get_first_appropriate_binding(&Path::new("/bin"), Host)
            .is_none()); // should correspond to no binding

        // testing symmetric binding
        fs.add_binding("/bin", "/bin").unwrap();

        assert_eq!(
            fs.get_first_appropriate_binding(&Path::new("/bin/folder/subfolder"), Guest)
                .unwrap()
                .get_path(Guest),
            &PathBuf::from("/bin")
        ); // it should detect the binding

        // assert_eq!(
        //     fs.get_first_appropriate_binding(&Path::new("/bin/folder/
        // subfolder"), Host)         .unwrap()
        //         .get_path(Guest),
        //     &PathBuf::from("/bin")
        // ); // same on the other side
    }

    #[test]
    fn test_fs_is_path_executable() {
        assert!(FileSystem::check_host_path_executable(&PathBuf::from("/bin/sleep")).is_ok());
        assert!(FileSystem::check_host_path_executable(&PathBuf::from("/../sleep")).is_err());
    }
}
