use crate::errors::*;
use crate::filesystem::binding::Side;
use crate::filesystem::FileSystem;
use nix::sys::stat::Mode;
use std::fs::FileType;
use std::path::{Path, PathBuf};

pub trait Substitutor {
    fn substitute(&self, path: &Path, from_side: Side) -> Result<PathBuf>;
    fn substitute_intermediary_and_glue(&self, path: &Path) -> Result<(PathBuf, Option<FileType>)>;
}

impl Substitutor for FileSystem {
    /// Finds a suitable binding for the given path,
    /// and changes its prefix from one side to another, if it can.
    ///
    /// Returns the substituted path,
    /// or `None` if the path wasn't modified.
    ///
    /// * `path` is the path that will be modified. Must be canonicalized.
    /// * `direction` is the direction of the substitution.
    #[inline]
    fn substitute(&self, path: &Path, from_side: Side) -> Result<PathBuf> {
        let maybe_binding = self.get_first_appropriate_binding(path, from_side);
        // TODO: should we substitute with root?
        if maybe_binding.is_none() {
            return Err(Error::errno_with_msg(
                ENOENT,
                format!(
                    "No binding found, when substituting binding for path: {:?}",
                    path
                ),
            ));
        }
        let binding = maybe_binding.unwrap();

        // Is it a "symmetric" binding?
        if !binding.needs_substitution() {
            return Ok(path.to_path_buf());
        }

        binding.substitute_path_prefix(path, from_side)
    }

    /// Substitute a binding of a canonicalized path, from `Guest` to `Host`,
    /// and uses glue if the user doesn't have the permissions necessary.
    ///
    /// The substituted path is returned along with its file type.
    #[inline]
    fn substitute_intermediary_and_glue(
        &self,
        guest_path: &Path,
    ) -> Result<(PathBuf, Option<FileType>)> {
        let host_path = self.substitute(guest_path, Side::Guest)?;

        // Retrieves the path's metadata without going through symlinks.
        match host_path.symlink_metadata().map_err(Error::from) {
            Ok(metadata) => Ok((host_path, Some(metadata.file_type()))),
            Err(_) => {
                if self.get_glue_type() != &Mode::empty() {
                    //TODO: implement glue

                    // TODO: maybe we can implement glue by return `Permission denied` when access
                    // glued path, instead of `mkdtemp`.

                    //        /* Build the glue between the hostfs and the guestfs during
                    //         * the initialization of a binding.  */
                    //        if (status < 0 && tracee->glue_type != 0) {
                    //            statl.st_mode = build_glue(tracee, guest_path, host_path,
                    // finality);            if (statl.st_mode == 0)
                    //                status = -1;
                    //        }

                    // for now we return the same path
                    Ok((host_path, None))
                } else {
                    Err(Error::errno_with_msg(
                        ENOENT,
                        "when substituting intermediary without glue",
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Error;
    use crate::filesystem::binding::Binding;
    use crate::filesystem::binding::Side::{Guest, Host};
    use crate::filesystem::FileSystem;
    use crate::utils::tests::get_test_rootfs_path;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_substitute_binding_root_and_asymmetric() {
        let rootfs_path = get_test_rootfs_path();
        let mut fs = FileSystem::with_root(&rootfs_path).unwrap();

        // "/etc" on the host, "/media" on the guest
        fs.add_binding(Binding::new("/etc", "/media", true));

        assert_eq!(
            fs.substitute(&Path::new("/../../../.."), Host),
            Err(Error::errno(ENOENT))
        ); // invalid path

        assert_eq!(
            fs.substitute(&Path::new("/etc/folder/subfolder"), Host),
            Ok(PathBuf::from("/media/folder/subfolder"))
        ); // "/etc" => "/media"

        assert_eq!(
            fs.substitute(&Path::new("/media/folder/subfolder"), Host,),
            Err(Error::errno(ENOENT))
        ); // the path isn't translatable to the guest fs (it's outside of the proot jail)

        assert_eq!(
            fs.substitute(&Path::new("/etc/folder/subfolder"), Guest),
            Ok(rootfs_path.join("etc/folder/subfolder"))
        ); // "/" => "/home/user"

        assert_eq!(
            fs.substitute(&Path::new("/media/folder/subfolder"), Guest,),
            Ok(PathBuf::from("/etc/folder/subfolder"))
        ); // "/media" => "/etc"
    }

    #[test]
    fn test_substitute_binding_symmetric() {
        let mut fs = FileSystem::with_root(get_test_rootfs_path()).unwrap();

        fs.add_binding(Binding::new("/etc/something", "/etc/something", true));

        let path = PathBuf::from("/etc/something/subfolder");

        assert_eq!(
            fs.substitute(&path, Guest),
            Ok(path.clone()) // the binding is symmetric
        );

        assert_eq!(
            fs.substitute(&path, Host),
            Ok(path.clone()) // same in the other direction
        );
    }

    #[test]
    fn test_substitute_intermediary_and_glue() {
        let rootfs_path = get_test_rootfs_path();
        let mut fs =
            FileSystem::with_root(PathBuf::from(rootfs_path.as_path()).join("bin")).unwrap();

        // testing a folder
        let (path, file_type) = fs
            .substitute_intermediary_and_glue(&Path::new("/sleep"))
            .expect("no error");

        assert_eq!(path, PathBuf::from(rootfs_path).join("bin/sleep")); // "/" => "/usr/bin/"
        assert!(file_type.unwrap().is_file());

        fs.add_binding(Binding::new("/bin", "/bin", true));

        // testing a symlink
        let (path_2, file_type_2) = fs
            .substitute_intermediary_and_glue(&Path::new("/bin/sh"))
            .expect("no error");

        assert_eq!(path_2, PathBuf::from("/bin/sh")); // no change in path, because symmetric binding
        assert!(file_type_2.unwrap().is_symlink());

        // testing a file
        let (path_3, file_type_3) = fs
            .substitute_intermediary_and_glue(&Path::new("/bin/true"))
            .expect("no error");

        assert_eq!(path_3, PathBuf::from("/bin/true")); // same here
        assert!(file_type_3.unwrap().is_file() || file_type_3.unwrap().is_symlink());
    }
}
