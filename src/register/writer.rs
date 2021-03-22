use byteorder::{LittleEndian, ReadBytesExt};
use errors::Result;
use libc::c_void;
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::{PTRACE_PEEKDATA, PTRACE_POKEDATA};
use register::reader::convert_word_to_bytes;
use register::{PtraceMemoryAllocator, Registers, SysArg, SysArgIndex, Word};
use std::io::Cursor;
use std::io::Read;
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr::null_mut;

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
    fn set_sysarg_path(
        &mut self,
        sys_arg: SysArgIndex,
        path: &Path,
        justification: &'static str,
    ) -> Result<()>;
    fn set_sysarg_data(
        &mut self,
        sys_arg: SysArgIndex,
        data: &[u8],
        justification: &'static str,
    ) -> Result<()>;
    fn write_data(&self, dest_tracee: *mut Word, data: &[u8]) -> Result<()>;
}

impl PtraceWriter for Registers {
    /// Converts `path` into bytes before calling the following function.
    fn set_sysarg_path(
        &mut self,
        sys_arg: SysArgIndex,
        path: &Path,
        justification: &'static str,
    ) -> Result<()> {
        self.set_sysarg_data(sys_arg, path.as_os_str().as_bytes(), justification)
    }

    /// Copies all bytes of `data` to the tracee's memory block
    /// and makes `sys_arg` point to this new block.
    fn set_sysarg_data(
        &mut self,
        sys_arg: SysArgIndex,
        data: &[u8],
        justification: &'static str,
    ) -> Result<()> {
        // Allocate space into the tracee's memory to host the new data.
        let tracee_ptr = self.alloc_mem(data.len() as isize)?;

        // Copy the new data into the previously allocated space.
        self.write_data(tracee_ptr as *mut Word, data)?;

        // Make this argument point to the new data.
        self.set(SysArg(sys_arg), tracee_ptr, justification);

        Ok(())
    }

    fn write_data(&self, dest_tracee: *mut Word, data: &[u8]) -> Result<()> {
        //TODO implement belongs_to_heap_prealloc
        // if (belongs_to_heap_prealloc(tracee, dest_tracee))
        // return -EFAULT;

        //TODO implement HAVE_PROCESS_VM

        // The byteorder crate is used to read the [u8] slice as a [Word] slice.
        let null_char_slice: &[u8] = &[b'\0'];
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
        for byte in bytes.iter_mut().take(nb_trailing_bytes as usize) {
            *byte = buf.read_u8().unwrap();
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
    use nix::unistd::execvp;
    use register::{Current, Original, PtraceReader, SysArg1};
    use sc::nr::MKDIR;
    use std::ffi::CString;
    use std::path::PathBuf;
    use utils::tests::fork_test;

    #[test]
    fn test_write_set_sysarg_path_write_same_path() {
        let test_path = "my/impossible/test/path";
        let test_path_2 = "my/second/impossible/test/path";

        fork_test(
            "/",
            // expecting an error (because the first path doesn't exit)
            1,
            // parent
            |tracee, _| {
                if tracee.regs.get_sys_num(Current) == MKDIR {
                    tracee.regs.set_restore_original_regs(false);
                    tracee.regs.save_current_regs(Original);

                    let dir_path = tracee.regs.get_sysarg_path(SysArg1).unwrap();

                    // we're checking that the string read in the tracee's memory
                    // corresponds to what has been given to the execve command
                    assert_eq!(dir_path, PathBuf::from(test_path));

                    // we write the new path
                    assert!(tracee
                        .regs
                        .set_sysarg_path(
                            SysArg1,
                            &PathBuf::from(test_path_2),
                            "setting impossible path for push_regs test",
                        )
                        .is_ok());

                    // we read the new path from the tracee's memory
                    let dir_path_2 = tracee.regs.get_sysarg_path(SysArg1).unwrap();

                    // the written and newly read paths must be the same
                    assert_eq!(dir_path_2, PathBuf::from(test_path_2));

                    // we don't push the regs, we stop here
                    true
                } else {
                    false
                }
            },
            // child
            || {
                // calling the mkdir function, which should call the MKDIR syscall
                execvp(
                    &CString::new("mkdir").unwrap(),
                    &[CString::new(".").unwrap(), CString::new(test_path).unwrap()],
                )
                .expect("failed execvp mkdir");
            },
        );
    }
}
