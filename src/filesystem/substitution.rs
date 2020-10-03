use errors::{Error, Result};
use filesystem::binding::Direction;
use filesystem::binding::Side::{Guest, Host};
use filesystem::FileSystem;
use nix::sys::stat::Mode;
use std::fs::FileType;
use std::path::{Path, PathBuf};

pub trait Substitutor {
    fn substitute_binding(&self, path: &Path, direction: Direction) -> Result<Option<PathBuf>>;
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
    fn substitute_binding(&self, path: &Path, direction: Direction) -> Result<Option<PathBuf>> {
        let maybe_binding = self.get_binding(path, direction.0);

        if maybe_binding.is_none() {
            return Err(Error::no_such_file_or_dir(
                "when substituting binding, no binding found",
            ));
        }
        let binding = maybe_binding.unwrap();

        // Is it a "symmetric" binding?
        if !binding.needs_substitution() {
            return Ok(None);
        }

        Ok(binding.substitute_path_prefix(path, direction)?)
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
        let substituted_path = self.substitute_binding(guest_path, Direction(Guest, Host))?;
        let host_path = substituted_path.unwrap_or(guest_path.to_path_buf());

        match self.get_direct_metadata(&host_path) {
            Ok(metadata) => Ok((host_path, Some(metadata.file_type()))),
            Err(_) => {
                if self.get_glue_type() != &Mode::empty() {
                    //TODO: implement glue
                    //        /* Build the glue between the hostfs and the guestfs during
                    //         * the initialization of a binding.  */
                    //        if (status < 0 && tracee->glue_type != 0) {
                    //            statl.st_mode = build_glue(tracee, guest_path, host_path, finality);
                    //            if (statl.st_mode == 0)
                    //                status = -1;
                    //        }

                    // for now we return the same path
                    Ok((host_path, None))
                } else {
                    Err(Error::no_such_file_or_dir(
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
    use errors::Error;
    use filesystem::binding::Binding;
    use filesystem::binding::Side::{Guest, Host};
    use filesystem::FileSystem;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_substitute_binding_root_and_asymmetric() {
        let mut fs = FileSystem::with_root("/home/user");

        // "/etc" on the host, "/media" on the guest
        fs.add_binding(Binding::new("/etc", "/media", true));

        assert_eq!(
            fs.substitute_binding(&Path::new("/../../../.."), Direction(Host, Guest)),
            Err(Error::no_such_file_or_dir(
                "when substituting binding, no binding found",
            ))
        ); // invalid path

        assert_eq!(
            fs.substitute_binding(&Path::new("/etc/folder/subfolder"), Direction(Host, Guest)),
            Ok(Some(PathBuf::from("/media/folder/subfolder")))
        ); // "/etc" => "/media"

        assert_eq!(
            fs.substitute_binding(
                &Path::new("/media/folder/subfolder"),
                Direction(Host, Guest),
            ),
            Err(Error::no_such_file_or_dir(
                "when substituting binding, no binding found",
            ))
        ); // the path isn't translatable to the guest fs (it's outside of the proot jail)

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
        let mut fs = FileSystem::with_root("/home/user");

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

    #[test]
    fn test_substitute_intermediary_and_glue() {
        let mut fs = FileSystem::with_root("/etc/acpi");

        // testing a folder
        let (path, file_type) = fs
            .substitute_intermediary_and_glue(&Path::new("/events"))
            .expect("no error");

        assert_eq!(path, PathBuf::from("/etc/acpi/events")); // "/" => "/etc/acpi/"
        assert!(file_type.unwrap().is_dir());

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
        assert!(file_type_3.unwrap().is_file());
    }
}
