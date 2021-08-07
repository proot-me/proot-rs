use crate::errors::Result;
use crate::filesystem::temp::TempFile;
use libc::{S_IRUSR, S_IXUSR};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const LOADER_EXE: &'static [u8] = include_bytes!("loader/loader-shim");

pub trait LoaderFile {
    fn prepare_loader(&self) -> Result<()>;
    fn get_loader_path(&self) -> &Path;
}

impl LoaderFile for TempFile {
    fn prepare_loader(&self) -> Result<()> {
        let mut file = self.create_file()?;
        let mut perms = file.metadata()?.permissions();

        // copy the binary loader in this temporary file
        file.write_all(LOADER_EXE)?;

        // make it readable and executable
        perms.set_mode((S_IRUSR | S_IXUSR) as _);
        file.set_permissions(perms)?;

        Ok(())
    }

    fn get_loader_path(&self) -> &Path {
        &self.path
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

            loader.prepare_loader().unwrap();

            // the loader must exist now
            assert!(loader_path.exists());

            loader_path
        };

        // the loader must have been deleted
        assert!(!loader_path.exists());
    }
}
