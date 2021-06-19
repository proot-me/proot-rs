use crate::errors::Result;

use crate::filesystem::binding::Side::{Guest, Host};
use crate::filesystem::canonicalization::Canonicalizer;
use crate::filesystem::substitution::Substitutor;
use crate::filesystem::FileSystem;
use std::path::{Path, PathBuf};

pub trait Translator {
    fn translate_path<P: AsRef<Path>>(&self, guest_path: P, deref_final: bool) -> Result<PathBuf>;
    fn translate_absolute_path<P: AsRef<Path>>(
        &self,
        guest_path: P,
        deref_final: bool,
    ) -> Result<PathBuf>;
    fn detranslate_path<P: AsRef<Path>>(
        &self,
        host_path: P,
        referrer: Option<&Path>,
    ) -> Result<Option<PathBuf>>;
}

impl Translator for FileSystem {
    /// Translates a path from `guest` to `host`. Relative guest path is also
    /// accepted.
    fn translate_path<P: AsRef<Path>>(&self, guest_path: P, deref_final: bool) -> Result<PathBuf> {
        if guest_path.as_ref().is_relative() {
            // It is relative to the current working directory.
            let mut absolute_guest_path = PathBuf::from(self.get_cwd());
            absolute_guest_path.push(guest_path);
            self.translate_absolute_path(&absolute_guest_path, deref_final)
        } else {
            self.translate_absolute_path(guest_path, deref_final)
        }
    }

    /// Translates a path from `guest` to `host`. Only absolute guest path is
    /// accepted.
    fn translate_absolute_path<P: AsRef<Path>>(
        &self,
        guest_path: P,
        deref_final: bool,
    ) -> Result<PathBuf> {
        let canonical_guest_path = self.canonicalize(&guest_path, deref_final)?;
        let host_path = self.substitute(&canonical_guest_path, Guest)?;
        Ok(host_path)
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
    fn detranslate_path<P: AsRef<Path>>(
        &self,
        host_path: P,
        referrer: Option<&Path>,
    ) -> Result<Option<PathBuf>> {
        let host_path = host_path.as_ref();
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
                let maybe_binding_referree = self.get_first_appropriate_binding(host_path, Host);
                let binding_referrer = self
                    .get_first_appropriate_binding(referrer_path, Host)
                    .unwrap();

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
            if let Ok(maybe_path) = self.substitute(host_path, Host) {
                // TODO: Error handling
                // if a suitable binding was found, we stop here
                return Ok(Some(maybe_path));
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

    use crate::filesystem::FileSystem;
    use crate::utils::tests::get_test_rootfs_path;
    use nix::sys::stat::Mode;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_translate_path_without_root() {
        let mut fs = FileSystem::with_root("/").unwrap();

        assert_eq!(
            fs.translate_path("/home/../etc/./../etc", false),
            Ok("/etc".into())
        ); // simple canonicalization here

        fs.add_binding("/etc", "/home").unwrap();

        assert_eq!(
            fs.translate_path(&Path::new("/home/passwd"), false),
            Ok(PathBuf::from("/etc/passwd"))
        );
    }

    #[test]
    fn test_translate_path_with_root() {
        let rootfs_path = get_test_rootfs_path();

        let mut fs = FileSystem::with_root(&rootfs_path).unwrap();

        assert_eq!(
            fs.translate_path("/bin/sleep", false),
            Ok(rootfs_path.clone().join("bin/sleep"))
        );

        fs.add_binding("/usr/bin", "/bin").unwrap();

        fs.set_glue_type(Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO);

        // "/bin/true" -> "/usr/bin/true"
        assert_eq!(
            fs.translate_path(&Path::new("/bin/true"), false),
            Ok(PathBuf::from("/usr/bin/true"))
        );

        // checking that the substitution only happens at the end ("/" is translated,
        // not "/bin")
        // "/bin/../home" -> "${rootfs}/bin/home"
        assert_eq!(
            fs.translate_path(&Path::new("/bin/../home"), false),
            Ok(PathBuf::from(&rootfs_path).join("home"))
        );
    }

    #[test]
    fn test_detranslate_path_root() {
        let rootfs_path = PathBuf::from(get_test_rootfs_path());

        // "${rootfs}" on the host, "/" on the guest
        let fs = FileSystem::with_root(rootfs_path.as_path()).unwrap();

        // "${rootfs}/bin/sleep" -> "/bin/sleep"
        assert_eq!(
            fs.detranslate_path(&PathBuf::from(&rootfs_path).join("bin/sleep"), None),
            Ok(Some(PathBuf::from("/bin/sleep")))
        );

        // "${rootfs}" -> "/"
        assert_eq!(
            fs.detranslate_path(&Path::new(rootfs_path.as_path()), None),
            Ok(Some(PathBuf::from("/")))
        );

        // "${rootfs}/home/other_user" -> "/home/other_user"
        assert_eq!(
            fs.detranslate_path(&PathBuf::from(&rootfs_path).join("home/other_user"), None),
            Ok(Some(PathBuf::from("/home/other_user")))
        );
    }

    #[test]
    fn test_detranslate_path_asymmetric() {
        let rootfs_path = get_test_rootfs_path();

        // "${rootfs}" on the host, "/" on the guest
        let mut fs = FileSystem::with_root(rootfs_path).unwrap();

        fs.add_binding("/etc", "/tmp").unwrap();

        assert_eq!(
            fs.detranslate_path(&Path::new("/etc/passwd"), None),
            Ok(Some(PathBuf::from("/tmp/passwd")))
        );
    }

    #[test]
    fn test_detranslate_path_symmetric() {
        // "${rootfs}" on the host, "/" on the guest
        let mut fs = FileSystem::with_root(get_test_rootfs_path()).unwrap();

        fs.add_binding("/etc", "/etc").unwrap();

        assert_eq!(
            fs.detranslate_path("/etc/guest/something", None),
            Ok(Some("/etc/guest/something".into()))
        ); // no change in path, because it's a symmetric binding

        //TODO: detranslate symlink tests
    }
}
