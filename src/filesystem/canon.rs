use std::path::{Path, PathBuf};
use nix::Result;
use nix::errno::Errno;
use nix::Error;
use filesystem::fsnamespace::FileSystemNamespace;

pub trait Canonicalizor {
    fn canonicalize(&self, path: &Path, deref_final: bool) -> Result<PathBuf>;
}

impl Canonicalizor for FileSystemNamespace {
    fn canonicalize(&self, path: &Path, deref_final: bool) -> Result<PathBuf> {
        match path.canonicalize() {
            Ok(canonicalized_path) => Ok(canonicalized_path),
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
    use std::path::PathBuf;
    use nix::Error;
    use nix::errno::Errno;
    use filesystem::fsnamespace::FileSystemNamespace;

    #[test]
    fn test_canonicalize_invalid_path() {
        let mut fs = FileSystemNamespace::new();

        // "/home/user" on the host, "/" on the guest
        fs.set_root("/home/user");

        let path = PathBuf::from("/../../../test");

        assert_eq!(
            fs.canonicalize(&path, false),
            Err(Error::Sys(Errno::ENOENT))
        );
    }

    #[test]
    fn test_canonicalize_impossible_path() {
        let mut fs = FileSystemNamespace::new();

        // "/home/user" on the host, "/" on the guest
        fs.set_root("/home/user");
        let path = PathBuf::from("/impossible/path/over/there");

        assert_eq!(
            fs.canonicalize(&path, false),
            Err(Error::Sys(Errno::ENOENT))
        );
    }

    #[test]
    fn test_canonicalize_normal_path() {
        let mut fs = FileSystemNamespace::new();

        // "/home/user" on the host, "/" on the guest
        fs.set_root("/home/user");

        let path = PathBuf::from("/home/../bin/./../bin/sleep");

        assert_eq!(
            fs.canonicalize(&path, false),
            Ok(PathBuf::from("/bin/sleep"))
        );
    }
}
