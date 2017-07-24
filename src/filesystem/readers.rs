use std::{slice, mem};
use std::fs::File;
use std::io::{Result, Read, Seek, SeekFrom};
use std::path::PathBuf;

pub trait ExtraReader {
    /// Reads the context of a file, and extracts + transmutes its content into a structure.
    fn read_struct<T>(&mut self) -> Result<T>;
    /// Reads a path of the given size at a given offset, while not moving the file cursor.
    fn pread_path_at(&mut self, path_size: usize, offset: u64) -> Result<PathBuf>;
}

impl ExtraReader for File {
    fn read_struct<T>(&mut self) -> Result<T> {
        let num_bytes = mem::size_of::<T>();
        unsafe {
            let mut s = mem::uninitialized();
            let mut buffer = slice::from_raw_parts_mut(&mut s as *mut T as *mut u8, num_bytes);
            match self.read_exact(buffer) {
                Ok(()) => Ok(s),
                Err(e) => {
                    ::std::mem::forget(s);
                    Err(e)
                }
            }
        }
    }

    fn pread_path_at(&mut self, path_size: usize, offset: u64) -> Result<PathBuf> {
        // save the initial position
        let initial_pos = self.seek(SeekFrom::Current(0)).unwrap();
        let mut buffer = vec![0; path_size];

        // move the cursor to the offset
        self.seek(SeekFrom::Start(offset))?;
        self.read_exact(&mut buffer)?;

        // restore the initial position
        self.seek(SeekFrom::Start(initial_pos))?;

        Ok(PathBuf::from(
            unsafe { String::from_utf8_unchecked(buffer) },
        ))
    }
}
