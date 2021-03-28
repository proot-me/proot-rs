use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};
use std::path::PathBuf;
use std::{mem, slice};

pub trait ExtraReader {
    fn read_struct<T>(&mut self) -> Result<T>;
    fn pread_path_at(&mut self, path_size: usize, offset: u64) -> Result<PathBuf>;
}

impl ExtraReader for File {
    /// Reads the context of a file, and extracts + transmutes its content into
    /// a structure.
    fn read_struct<T>(&mut self) -> Result<T> {
        let num_bytes = mem::size_of::<T>();
        unsafe {
            let mut s = mem::MaybeUninit::uninit();
            let buffer = slice::from_raw_parts_mut(s.as_mut_ptr() as *mut u8, num_bytes);
            match self.read_exact(buffer) {
                Ok(()) => Ok(s.assume_init()),
                Err(e) => {
                    ::std::mem::forget(s);
                    Err(e)
                }
            }
        }
    }

    /// Reads a path of the given size at a given offset, while not moving the
    /// file cursor.
    ///
    /// The file's cursor is reinitialised to its initial position afterwards
    /// (simulates `pread`).
    ///
    /// `path_size` is the number of bytes that will be read on the file.
    /// `offset` is the starting point of the read.
    fn pread_path_at(&mut self, path_size: usize, offset: u64) -> Result<PathBuf> {
        // save the initial position
        let initial_pos = self.seek(SeekFrom::Current(0)).unwrap();
        let mut buffer = vec![0; path_size];

        // move the cursor to the offset
        self.seek(SeekFrom::Start(offset))?;
        self.read_exact(&mut buffer)?;

        // restore the initial position
        self.seek(SeekFrom::Start(initial_pos))?;

        Ok(PathBuf::from(unsafe {
            String::from_utf8_unchecked(buffer)
        }))
    }
}
