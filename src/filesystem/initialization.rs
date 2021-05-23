use crate::errors::Result;
use crate::filesystem::{Canonicalizer, FileSystem};
use nix::unistd::getcwd;
use std::path::PathBuf;

pub trait Initialiser {
    fn initialize(&mut self) -> Result<()>;
    fn initialize_cwd(&mut self) -> Result<()>;
}

impl Initialiser for FileSystem {
    fn initialize(&mut self) -> Result<()> {
        self.initialize_cwd()?;
        Ok(())
    }

    #[inline]
    fn initialize_cwd(&mut self) -> Result<()> {
        // Prepare the base in case cwd is relative.
        let mut raw_cwd = match self.get_cwd().is_relative() {
            // FIXME: This will crash when get_cwd() is a relative path. Because
            // nix::unistd::getcwd() returns a host path, which will result in `raw_cwd`
            // also being a host path. This problem also exists in proot written in C.
            true => getcwd()?,
            false => PathBuf::new(),
        };

        raw_cwd.push(self.get_cwd());
        // Ensures canonicalize() will report an error
        // if raw_cwd doesn't exist or isn't a directory.
        raw_cwd.push(PathBuf::from("."));

        let cwd = match self.canonicalize(&raw_cwd, true) {
            Ok(path) => path,
            Err(err) => {
                //TODO: log error
                eprintln!(
                    "proot warning: can't chdir (\"{}\") in the guest rootfs: {}",
                    raw_cwd.display(),
                    err
                );
                println!("proot info: default working directory is now \"/\"");
                PathBuf::from("/")
            }
        };

        // Replace with the canonicalized working directory.
        self.set_cwd(cwd);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::FileSystem;
    use crate::utils::tests::get_test_rootfs;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_initialisation_cwd_invalid_should_default_to_root() {
        let mut fs = FileSystem::with_root(get_test_rootfs());

        fs.set_cwd(PathBuf::from("/my/impossible/cwd"));

        assert_eq!(Ok(()), fs.initialize_cwd());

        // because the `cwd` we set is not existed, the cwd must have defaulted to "/"
        assert!(fs.get_cwd().is_absolute());
        assert!(fs.get_cwd().exists());
        assert_eq!(&PathBuf::from("/"), fs.get_cwd());
    }

    #[test]
    fn test_initialisation_cwd_absolute() {
        let mut fs = FileSystem::with_root(get_test_rootfs());

        fs.set_cwd(PathBuf::from("/bin"));

        assert_eq!(Ok(()), fs.initialize_cwd());

        // because the value of cwd is `/bin`, and ${rootfs}/bin exists, so the cwd need
        // not to be reset to "/".
        assert!(fs.get_cwd().is_absolute());
        assert!(fs.get_cwd().exists());
        assert_eq!(&PathBuf::from("/bin"), fs.get_cwd());
    }

    #[test]
    fn test_initialisation_cwd_relative() {
        let rootfs_path = get_test_rootfs();
        let mut fs = FileSystem::with_root(rootfs_path.as_path());
        // let real_cwd = getcwd().unwrap();

        fs.set_cwd(PathBuf::from("./.."));

        // the cwd should be reset to default value "/"
        assert_eq!(Ok(()), fs.initialize_cwd());

        assert!(fs.get_cwd().is_absolute());
        assert_eq!(Path::new("/"), fs.get_cwd());
    }
}
