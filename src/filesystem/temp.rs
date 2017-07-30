use std::env;
use std::path::PathBuf;
use std::fs;
use nix::unistd::getpid;
use errors::Result;

#[derive(Debug)]
pub struct TempFile {
    pub path: PathBuf,
}

impl TempFile {
    #[inline]
    fn create_temp_path(prefix: &str) -> PathBuf {
        PathBuf::from(format!(
            "{}/{}-{}-XXXXXX",
            env::temp_dir().to_str().unwrap(),
            prefix,
            getpid()
        ))
    }

    pub fn new(prefix: &str) -> Self {
        Self { path: TempFile::create_temp_path(prefix) }
    }

    pub fn get_file(&self) -> Result<fs::File> {
        Ok(fs::File::create(&self.path)?)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if self.path.exists() {
            fs::remove_file(&self.path).expect("delete temp file");
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_temp_file_has_correct_path() {
        let temp_file = TempFile::new("test-1");

        assert!(temp_file.path.is_absolute());
        assert!(!temp_file.path.exists());
        assert!(temp_file.path.parent().unwrap().eq(&env::temp_dir()));
    }

    #[test]
    fn test_temp_file_is_created_and_deleted() {
        let temp_file_path = {
            let temp_file = TempFile::new("test-1");
            let temp_file_path = temp_file.path.to_owned();

            // the file must not exist before creating the file
            assert!(!temp_file_path.exists());

            {
                let file = temp_file.get_file().unwrap();

                // the file must have been created and must exist
                assert!(temp_file_path.exists());
            }

            // it must persist even after the File is dropped
            assert!(temp_file_path.exists());

            temp_file_path
        };

        // but it must be deleted when the TempFile is dropped (so when proot-rs stops)
        assert!(!temp_file_path.exists());
    }
}
