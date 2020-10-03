use errors::Result;
use filesystem::{Canonicalizer, FileSystem};
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
    use filesystem::FileSystem;
    use std::path::PathBuf;

    #[test]
    fn test_initialisation_cwd_invalid_should_default_to_root() {
        let mut fs = FileSystem::with_root("/");

        fs.set_cwd(PathBuf::from("/my/impossible/cwd"));

        assert_eq!(Ok(()), fs.initialize_cwd());

        // the cwd must have defaulted to "/"
        assert!(fs.get_cwd().is_absolute());
        assert!(fs.get_cwd().exists());
        assert_eq!(&PathBuf::from("/"), fs.get_cwd());
    }

    #[test]
    fn test_initialisation_cwd_absolute() {
        let mut fs = FileSystem::with_root("/");

        fs.set_cwd(PathBuf::from("/etc/acpi"));

        assert_eq!(Ok(()), fs.initialize_cwd());

        assert!(fs.get_cwd().is_absolute());
        assert!(fs.get_cwd().exists());
        assert_eq!(&PathBuf::from("/etc/acpi"), fs.get_cwd());
    }

    #[test]
    fn test_initialisation_cwd_relative() {
        let mut fs = FileSystem::with_root("/");
        let real_cwd = getcwd().unwrap();

        fs.set_cwd(PathBuf::from("./.."));

        // the cwd should be canonicalized and verified
        assert_eq!(Ok(()), fs.initialize_cwd());

        assert!(fs.get_cwd().is_absolute());
        assert!(fs.get_cwd().exists());
        assert_eq!(real_cwd.parent().unwrap(), fs.get_cwd());
    }
}
