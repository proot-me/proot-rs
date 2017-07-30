use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use libc::{S_IRUSR, S_IXUSR};
use errors::Result;
use filesystem::temp::TempFile;

const LOADER_EXE: &'static [u8] = include_bytes!("loader/binary_loader_exe");

pub trait LoaderFile {
    fn prepare_loader(&self) -> Result<()>;
}

impl LoaderFile for TempFile {
    fn prepare_loader(&self) -> Result<()> {
        let mut file = self.get_file()?;
        let mut perms = file.metadata()?.permissions();

        file.write_all(LOADER_EXE)?;
        perms.set_mode(S_IRUSR | S_IXUSR);
        file.set_permissions(perms)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_is_loaded_and_deleted() {
        let loader_path = {
            let loader = TempFile::new("prefix_test_loader_is_loaded");
            let loader_path = loader.path.to_owned();

            // the loader doesn't exist yet
            assert!(!loader_path.exists());

            loader.prepare_loader();

            // the loader must exist now
            assert!(loader_path.exists());

            loader_path
        };

        // the loader must have been deleted
        assert!(!loader_path.exists());
    }
}
