use std::path::{Path, PathBuf};
use std::os::unix::ffi::OsStrExt;
use std::mem;
use std::ptr::null_mut;
use std::io::Read;
use libc::c_void;
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::{PTRACE_POKEDATA, PTRACE_PEEKDATA};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use errors::Result;
use register::{Word, SysArgIndex, PtraceMemoryAllocator, Registers};
use register::reader::convert_word_to_bytes;

#[cfg(target_pointer_width = "32")]
#[inline]
pub fn convert_bytes_to_word(value_to_convert: [u8; 4]) -> Word {
    unsafe { mem::transmute(value_to_convert) }
}

#[cfg(target_pointer_width = "64")]
#[inline]
pub fn convert_bytes_to_word(value_to_convert: [u8; 8]) -> Word {
    unsafe { mem::transmute(value_to_convert) }
}

pub trait PtraceWriter {
    fn set_sysarg_path(&mut self, sys_arg: SysArgIndex, path: &Path) -> Result<()>;
    fn set_sysarg_data(&mut self, sys_arg: SysArgIndex, data: &[u8]) -> Result<()>;
    fn write_data(&self, dest_tracee: *mut Word, data: &[u8]) -> Result<()>;
}

impl PtraceWriter for Registers {
    /// Converts `path` into bytes before calling the following function.
    fn set_sysarg_path(&mut self, sys_arg: SysArgIndex, path: &Path) -> Result<()> {
        self.set_sysarg_data(sys_arg, path.as_os_str().as_bytes())
    }

    /// Copies all bytes of `data` to the tracee's memory block
    /// and makes `sys_arg` point to this new block.
    fn set_sysarg_data(&mut self, sys_arg: SysArgIndex, data: &[u8]) -> Result<()> {
        // Allocate space into the tracee's memory to host the new data.
        let tracee_ptr = self.alloc_mem(data.len() as isize)?;

        // Copy the new data into the previously allocated space.
        self.write_data(tracee_ptr as *mut Word, data)?;

        // Make this argument point to the new data.
        self.set_arg(sys_arg, tracee_ptr);

        Ok(())
    }

    fn write_data(&self, dest_tracee: *mut Word, data: &[u8]) -> Result<()> {
        //TODO implement belongs_to_heap_prealloc
        // if (belongs_to_heap_prealloc(tracee, dest_tracee))
        // return -EFAULT;

        //TODO implement HAVE_PROCESS_VM

        // The byteorder crate is used to read the [u8] slice as a [Word] slice.
        let null_char_slice: &[u8] = &['\0' as u8];
        let mut buf = Cursor::new(data).chain(Cursor::new(null_char_slice));

        let size = data.len() + 1; // the +1 is for the `\0` byte that we will have manually
        let word_size = mem::size_of::<Word>();
        let nb_trailing_bytes = (size % word_size) as isize;
        let nb_full_words = ((size - nb_trailing_bytes as usize) / word_size) as isize;

        // Copy one word by one word, except for the last one.
        for i in 0..nb_full_words {
            let word = buf.read_uint::<LittleEndian>(word_size).unwrap() as Word;
            let dest_addr = unsafe { dest_tracee.offset(i) as *mut c_void };

            ptrace(
                PTRACE_POKEDATA,
                self.get_pid(),
                dest_addr,
                word as *mut c_void,
            )?;
        }

        // Copy the bytes in the last word carefully since we have to
        // overwrite only the relevant ones.
        let last_dest_addr = unsafe { dest_tracee.offset(nb_full_words) as *mut c_void };
        let existing_word =
            ptrace(PTRACE_PEEKDATA, self.get_pid(), last_dest_addr, null_mut())? as Word;
        let mut bytes = convert_word_to_bytes(existing_word);

        // The trailing bytes are merged with the existing bytes. For example:
        // bytes = [0, 0, 0, 0, 0, 0, 119, 0] // the already existing bytes at the dest addr
        // trailing bytes = [164, 247, 274] // our trailing bytes
        // fusion = [164, 247, 274, 0, 0, 0, 119, 0] // the fusion of the two
        for j in 0..nb_trailing_bytes as usize {
            bytes[j] = buf.read_u8().unwrap();
        }

        let last_word = convert_bytes_to_word(bytes);
        // We can now safely write the final word.
        ptrace(
            PTRACE_POKEDATA,
            self.get_pid(),
            last_dest_addr,
            last_word as *mut c_void,
        )?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use nix::unistd::execvp;
    use utils::tests::fork_test;
    use syscall::nr::MKDIR;
    use register::PtraceReader;

    #[test]
    fn test_write_set_sysarg_path_write_same_path() {
        let test_path = "my/impossible/test/path";
        let test_path_2 = "my/second/impossible/test/path";

        fork_test(
            // expecting an error (because the path doesn't exit)
            1,
            // parent
            |regs, _, _| {
                if regs.sys_num == MKDIR {
                    let dir_path = regs.get_sysarg_path(SysArgIndex::SysArg1).unwrap();

                    // we're checking that the string read in the tracee's memory
                    // corresponds to what has been given to the execve command
                    assert_eq!(dir_path, PathBuf::from(test_path));

                    // we write the new path
                    assert!(
                        regs.set_sysarg_path(SysArgIndex::SysArg1, &PathBuf::from(test_path_2))
                            .is_ok()
                    );

                    // we read the new path from the tracee's memory
                    let dir_path_2 = regs.get_sysarg_path(SysArgIndex::SysArg1).unwrap();

                    // the written and newly read paths must be the same
                    assert_eq!(dir_path_2, PathBuf::from(test_path_2));

                    //TODO: push regs when implemented

                    // we can stop here
                    return true;
                } else {
                    return false;
                }
            },
            // child
            || {
                // calling the mkdir function, which should call the MKDIR syscall
                execvp(
                    &CString::new("mkdir").unwrap(),
                    &[CString::new(".").unwrap(), CString::new(test_path).unwrap()],
                ).expect("failed execvp mkdir");
            },
        );
    }
}
