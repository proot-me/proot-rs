use std::path::{Path, PathBuf};

use nix::sys::{self, stat::Mode};
use nix::unistd::{self, AccessFlags};

use crate::errors::*;
use crate::filesystem::binding::Side::Host;
use crate::filesystem::binding::{Binding, Side};

use super::{Canonicalizer, Substitutor, Translator};

/// The file-system information associated with one or more tracee, which
/// corresponds to the [`fs_struct`] structure in the kernel. If clone() is
/// called with `CLONE_FS` set, then both parent tracee and child tracee will
/// share this structure, otherwise a copy will be created.
///
/// [`fs_struct`]: https://elixir.bootlin.com/linux/latest/source/include/linux/fs_struct.h
#[derive(Debug, Clone)]
pub struct FileSystem {
    /// List of bindings used to replicate `mount` and `bind`.
    /// It will also contain the root binding (to replicate `chroot`).
    ///
    /// FIXME: Actually, bindings should not be part of the `fs_struct`, it
    /// should be shared globally
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
        // We need to ensure that the target path for the binding exists.
        // Skip the check for "/" because "/" always exists.
        if canonical_guest_path != Path::new("/") {
            self.substitute(&canonical_guest_path, Side::Guest)?
                .metadata()?; // call .metadata() to check if the path exist
        }

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
    /// `guest_path` should be an absolute path, because passing a relative path
    /// to `set_cwd()` can be very odd, especially when initializing proot-rs
    /// (when there is no cwd yet). In addition, the path must be a directory
    /// and have execute permissions.
    pub fn set_cwd<P: AsRef<Path>>(&mut self, guest_path: P) -> Result<()> {
        let guest_path = guest_path.as_ref();
        if guest_path.is_relative() {
            return Err(Error::errno_with_msg(
                Errno::EINVAL,
                format!(
                    "current work directory should at least not be a relative path: {:?}",
                    guest_path
                ),
            ));
        }

        let (canonical_guest_path, host_path) = self.translate_absolute_path(guest_path, true)?;

        // To change cwd to a dir, the tracee must have execute (`x`) permission to this
        // dir, FIXME: This may be wrong, because we need to check if tracee has
        // permission
        if !host_path.metadata()?.is_dir() {
            return Err(Error::errno(Errno::ENOTDIR));
        }
        nix::unistd::access(&host_path, AccessFlags::X_OK)?;

        self.cwd = canonical_guest_path;
        Ok(())
    }

    /// Set root directory for this FileSystem instance.
    /// The root path needs to be a host side path, and relative path are also
    /// accepted.
    #[inline]
    pub fn set_root<P: AsRef<Path>>(&mut self, host_path: P) -> Result<()> {
        let raw_root = host_path.as_ref();
        // the `root` is host path, we use host side canonicalize() to canonicalize it.
        // std::fs::canonicalize() also throws an error if the path does not exist.
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

    /// This function provides a way to check whether a path is canonical.
    ///
    /// NOTE: This check **is not a strict check**. This function does not
    /// dereference the final component of `path`, to allow for the case
    /// where the file does not exist but it's parent dir exists.
    #[cfg(test)]
    pub fn is_path_canonical<P: AsRef<Path>>(&self, path: P, side: Side) -> bool {
        let path = path.as_ref();
        if path.is_relative() {
            return false;
        }
        match side {
            Side::Host => {
                if let Ok(canonical_path) = std::fs::canonicalize(path) {
                    return canonical_path.as_os_str() == path.as_os_str();
                }
                // Since `std::fs::canonicalize()` may fail because `path` does not exist, we
                // will check its parent directory again.
                let parent_path = path.parent();
                match parent_path {
                    Some(parent_path) => {
                        let canonical_path = std::fs::canonicalize(parent_path);
                        canonical_path.is_ok()
                            && canonical_path.unwrap().as_os_str() == parent_path.as_os_str()
                    }
                    None => true, // `path` is "/", so we just return true in this case
                }
            }
            Side::Guest => {
                let canonical_path = self.canonicalize(path, false);
                canonical_path.is_ok() && canonical_path.unwrap().as_os_str() == path.as_os_str()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::binding::Side::{Guest, Host};
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

    /// Unit test for `FileSystem::is_path_canonical()`
    #[test]
    fn test_fs_is_path_canonical() -> Result<()> {
        let root_path = get_test_rootfs_path();
        let mut fs = FileSystem::with_root(root_path)?;
        fs.add_binding("/etc", "/bin")?;
        fs.set_cwd("/")?;

        assert_eq!(fs.is_path_canonical("", Side::Host), false);
        assert_eq!(fs.is_path_canonical("etc", Side::Host), false);
        assert_eq!(fs.is_path_canonical("/etc", Side::Host), true);
        assert_eq!(fs.is_path_canonical("/etc/", Side::Host), false);

        assert_eq!(fs.is_path_canonical("/etc/", Side::Guest), false);
        // `/lib64` is a symlink, which is not canonical when it is in the middle of the
        // path.
        assert_eq!(fs.is_path_canonical("/lib64/libc.so.6", Side::Guest), false);
        assert_eq!(fs.is_path_canonical("", Side::Guest), false);
        assert_eq!(fs.is_path_canonical("etc", Side::Guest), false);
        assert_eq!(fs.is_path_canonical("/etc/./", Side::Guest), false);
        assert_eq!(fs.is_path_canonical("/etc/../home", Side::Guest), false);
        assert_eq!(fs.is_path_canonical("../home", Side::Guest), false);

        Ok(())
    }

    /// Unit test for initialization functions in `FileSystem`(e.g. `set_cwd()`,
    /// `with_root()`, `add_binding()`)
    #[test]
    fn test_fs_init_functions() -> Result<()> {
        // should be ok.
        let fs = FileSystem::with_root("/etc/../")?;
        assert!(fs.is_path_canonical(fs.get_root(), Side::Host));
        assert_eq!(fs.get_root(), Path::new("/"));

        // will failed, root path must exist.
        FileSystem::with_root("/impossible_path").unwrap_err();

        // should be ok, host side relative path is accepted as input.
        let fs = FileSystem::with_root(".")?;
        assert!(fs.is_path_canonical(fs.get_root(), Side::Host));

        let root_path = get_test_rootfs_path();
        let mut fs = FileSystem::with_root(root_path)?;
        // we currently cannot bind to a non-existing guest path.
        fs.add_binding("/etc", "/bin/non_existing_path")
            .unwrap_err();
        fs.add_binding("/non_existing_path", "/bin").unwrap_err();
        fs.add_binding("/etc", "/usr")?;
        fs.add_binding("/etc/../tmp/", "/home/../home")?;
        fs.add_binding("home", "/home").unwrap_err();
        // should be failed since `guest_path` is not absolute path
        fs.add_binding("/dev", "tmp").unwrap_err();

        // path in binding should be canonical
        for binding in &fs.bindings {
            assert!(fs.is_path_canonical(binding.get_path(Side::Guest), Side::Guest));
            assert!(fs.is_path_canonical(binding.get_path(Side::Host), Side::Host));
        }

        // should be failed since `guest_path` is not absolute path
        fs.set_cwd(".").unwrap_err();
        fs.set_cwd("/")?;
        assert_eq!(&fs.cwd, Path::new("/"));
        fs.set_cwd("/../")?;
        assert_eq!(&fs.cwd, Path::new("/"));
        fs.set_cwd("/../etc/")?;
        assert_eq!(&fs.cwd, Path::new("/etc"));
        // should be failed since "/bin/ls" is not a dir
        fs.set_cwd("/bin/ls").unwrap_err();
        fs.set_cwd("/etc/passwd").unwrap_err();

        Ok(())
    }
}
