use std::path::{Path, PathBuf};
use errors::Result;
use filesystem::binding::Direction;
use filesystem::binding::Side::{Host, Guest};
use filesystem::FileSystem;
use filesystem::substitution::Substitutor;
use filesystem::canonicalization::Canonicalizer;

pub trait Translator {
    fn translate_path(&self, path: &Path, deref_final: bool) -> Result<PathBuf>;
    fn detranslate_path(&self, path: &Path, referrer: Option<&Path>) -> Result<Option<PathBuf>>;
}

impl Translator for FileSystem {
    /// Translates a path from `guest` to `host`.
    fn translate_path(&self, user_path: &Path, deref_final: bool) -> Result<PathBuf> {
        let mut guest_path = PathBuf::new();
        let user_path_is_absolute = user_path.is_absolute();

        if !user_path_is_absolute {
            // It is relative to the current working directory.
            guest_path.push(self.get_cwd().to_path_buf());

            //TODO: dir_fd != AT_FDCWD
        } else {
            guest_path.push(PathBuf::from("/"))
        }

        #[cfg(not(test))]
        println!(
            "\t translate({} + {})",
            guest_path.display(),
            user_path.display()
        );

        //TODO: log verbose
        // VERBOSE(tracee, 2, "pid %d: translate(\"%s\" + \"%s\")",
        //         tracee != NULL ? tracee->pid : 0, result, user_path);

        //TODO: event GUEST_PATH for extensions
        //    status = notify_extensions(tracee, GUEST_PATH, (intptr_t) result, (intptr_t) user_path);
        //    if (status < 0)
        //        return status;
        //    if (status > 0)
        //        goto skip;

        guest_path.push(user_path);
        guest_path = self.canonicalize(&guest_path, deref_final)?;
        let host_path = self.substitute_binding(&guest_path, Direction(Guest, Host))?;
        let result = host_path.unwrap_or(guest_path);

        #[cfg(not(test))]
        println!("\t\t -> {}", result.display());

        //TODO: log verbose
        // VERBOSE(tracee, 2, "pid %d:          -> \"%s\"",
        //         tracee != NULL ? tracee->pid : 0, result);

        Ok(result)
    }

    /// Translates a path from `host` to `guest`.
    ///
    /// `path` must canonicalized;
    /// Removes/substitutes the leading part of a "translated" `path`.
    ///
    /// Returns
    /// * `Ok(None)` if no translation is required (ie. symmetric binding).
    /// * `Ok(PathBuf)` is the path was translated.
    /// * An error otherwise.
    fn detranslate_path(
        &self,
        host_path: &Path,
        referrer: Option<&Path>,
    ) -> Result<Option<PathBuf>> {
        // Don't try to detranslate relative paths (typically
        // the target of a relative symbolic link).
        if host_path.is_relative() {
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
                let maybe_binding_referree = self.get_binding(host_path, Host);
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
            if let Ok(maybe_path) = self.substitute_binding(host_path, Direction(Host, Guest)) {
                // if a suitable binding was found, we stop here
                return Ok(maybe_path);
            }
        }

        // otherwise, we simply try to strip the (guest) root
        if let Ok(stripped_path) = host_path.strip_prefix(&self.get_root()) {
            return Ok(Some(PathBuf::from("/").join(stripped_path)));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use nix::sys::stat::{S_IRWXU, S_IRWXG, S_IRWXO};
    use filesystem::binding::Binding;
    use filesystem::FileSystem;

    #[test]
    fn test_translate_path_without_root() {
        let mut fs = FileSystem::with_root("/");

        assert_eq!(
            fs.translate_path(&Path::new("/home/../bin/./../bin"), false),
            Ok(PathBuf::from("/bin"))
        ); // simple canonicalization here

        // "/etc/host" in the host, "/etc/guest" in the guest
        fs.add_binding(Binding::new("/etc/acpi", "/home/test", true));

        assert_eq!(
            fs.translate_path(&Path::new("/home/test/events"), false),
            Ok(PathBuf::from("/etc/acpi/events"))
        ); // "/home/test" -> "/etc/acpi"
    }

    #[test]
    fn test_translate_path_with_root() {
        let mut fs = FileSystem::with_root("/etc/acpi");

        assert_eq!(
            fs.translate_path(&Path::new("/events"), false),
            Ok(PathBuf::from("/etc/acpi/events"))
        ); // "/home/test" -> "/etc/acpi"

        fs.add_binding(Binding::new("/usr/bin", "/bin", true));

        // necessary, because "/bin/true" probably doesn't exist in "/etc/acpi"
        fs.set_glue_type(S_IRWXU | S_IRWXG | S_IRWXO);

        assert_eq!(
            fs.translate_path(&Path::new("/bin/true"), false),
            Ok(PathBuf::from("/usr/bin/true"))
        ); // "/bin" -> "/usr/bin"

        assert_eq!(
            fs.translate_path(&Path::new("/bin/../home"), false),
            Ok(PathBuf::from("/etc/acpi/home"))
        ); // checking that the substitution only happens at the end ("/" is translated, not "/bin")
    }

    #[test]
    fn test_detranslate_path_root() {
        // "/home/user" on the host, "/" on the guest
        let fs = FileSystem::with_root("/home/user");

        assert_eq!(
            fs.detranslate_path(&Path::new("/home/user/bin/sleep"), None),
            Ok(Some(PathBuf::from("/bin/sleep")))
        ); // "/home/user" -> "/"

        assert_eq!(
            fs.detranslate_path(&Path::new("/home/user"), None),
            Ok(Some(PathBuf::from("/")))
        ); // "/home/user" -> "/"

        assert_eq!(
            fs.detranslate_path(&Path::new("/home/user/home/other_user"), None),
            Ok(Some(PathBuf::from("/home/other_user")))
        ); // "/home/user" -> "/"
    }
    #[test]
    fn test_detranslate_path_asymmetric() {
        // "/home/user" on the host, "/" on the guest
        let mut fs = FileSystem::with_root("/home/user");

        // "/etc/host" in the host, "/etc/guest" in the guest
        fs.add_binding(Binding::new("/etc/host", "/etc/guest", true));

        assert_eq!(
            fs.detranslate_path(&Path::new("/etc/host/something"), None),
            Ok(Some(PathBuf::from("/etc/guest/something")))
        ); // "/etc/host" -> "/etc/guest"
    }

    #[test]
    fn test_detranslate_path_symmetric() {
        // "/home/user" on the host, "/" on the guest
        let mut fs = FileSystem::with_root("/home/user");

        // "/etc/host" in the host, "/etc/guest" in the guest
        fs.add_binding(Binding::new("/etc/guest", "/etc/guest", true));

        assert_eq!(
            fs.detranslate_path(&Path::new("/etc/guest/something"), None),
            Ok(None)
        ); // no change in path, because it's a symmetric binding

        //TODO: detranslate symlink tests
    }
}
