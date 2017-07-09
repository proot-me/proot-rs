use std::path::{Path, PathBuf};
use nix::Result;
use filesystem::binding::Direction;
use filesystem::binding::Side::{Host, Guest};
use filesystem::fsnamespace::FileSystemNamespace;
use filesystem::substitution::Substitutor;

pub trait Translator {
    fn translate_path(&self, path: &Path) -> Result<PathBuf>;
    fn detranslate_path(&self, path: &Path, referrer: Option<&Path>) -> Result<Option<PathBuf>>;
}

impl Translator for FileSystemNamespace {
    /// Translates a path from `guest` to `host`.
    /// Remove/substitute the leading part of a "translated" `path`.
    ///
    /// Returns
    /// * `Ok(None)` if no translation is required (ie. symmetric binding).
    /// * `Ok(PathBuf)` is the path was translated.
    /// * An error otherwise.
    fn detranslate_path(&self, path: &Path, referrer: Option<&Path>) -> Result<Option<PathBuf>> {
        // Don't try to detranslate relative paths (typically
        // the target of a relative symbolic link).
        if path.is_relative() {
            return Ok(None);
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
                    follow_binding = binding_referree.get_path(Host) ==
                        binding_referrer.get_path(Host);
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
        if let Ok(stripped_path) = path.strip_prefix(&self.get_root()) {
            return Ok(Some(PathBuf::from("/").join(stripped_path)));
        }

        Ok(None)
    }

    fn translate_path(&self, user_path: &Path) -> Result<PathBuf> {
        let mut result = PathBuf::new();
        let user_path_is_absolute = user_path.is_absolute();

        if !user_path_is_absolute {
            // It is relative to the current working directory.
            result.push(self.get_cwd().to_path_buf());

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use filesystem::binding::Binding;
    use filesystem::fsnamespace::FileSystemNamespace;

    #[test]
    fn test_detranslate_path_root() {
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
    }

    #[test]
    fn test_detranslate_path_asymmetric() {
        let mut fs = FileSystemNamespace::new();

        // "/home/user" on the host, "/" on the guest
        fs.set_root("/home/user");

        // "/etc/host" in the host, "/etc/guest" in the guest
        fs.add_binding(Binding::new("/etc/host", "/etc/guest", true));

        assert_eq!(
            fs.detranslate_path(&Path::new("/etc/guest/something"), None),
            Ok(Some(PathBuf::from("/etc/host/something")))
        ); //

        //TODO: detranslate symlink tests
    }

    #[test]
    fn test_detranslate_path_symmetric() {
        let mut fs = FileSystemNamespace::new();

        // "/home/user" on the host, "/" on the guest
        fs.set_root("/home/user");

        // "/etc/host" in the host, "/etc/guest" in the guest
        fs.add_binding(Binding::new("/etc/host", "/etc/guest", true));

        assert_eq!(
            fs.detranslate_path(&Path::new("/etc/guest/something"), None),
            Ok(Some(PathBuf::from("/etc/host/something")))
        ); //

        //TODO: detranslate symlink tests
    }

    /*
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
