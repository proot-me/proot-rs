use std::{slice, mem};
use std::fs::File;
use std::io::{Result, Read};

pub trait StructReader {
    /// Reads the context of a file, and extracts + transmutes its content into a structure.
    fn read_struct<T>(&mut self) -> Result<T>;
}

impl StructReader for File {
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
}