use crate::errors::Result;
use crate::register::reader::convert_word_to_bytes;
use crate::register::{PtraceMemoryAllocator, Registers, SysArg, SysArgIndex, Word};
use byteorder::NativeEndian;
use byteorder::ReadBytesExt;
use libc::c_void;
use nix::sys::ptrace;
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
        append_null: bool,
    ) -> Result<()>;
    fn write_data(&self, dest_tracee: *mut Word, data: &[u8], append_null: bool) -> Result<()>;
}

impl PtraceWriter for Registers {
    /// Copy the `path` to tracee's memory space, and make the register
    /// `sys_arg` point to it. A null byte (b'\0') is implicitly appended to the
    /// end.
    ///
    /// Note that this will "allocate" a block of memory on stack, which means
    /// the value of the stack pointer register will be implicitly modified.
    fn set_sysarg_path(
        &mut self,
        sys_arg: SysArgIndex,
        path: &Path,
        justification: &'static str,
    ) -> Result<()> {
        self.set_sysarg_data(sys_arg, path.as_os_str().as_bytes(), justification, true)
    }

    /// Copy the `data` to tracee's memory space, and make the register
    /// `sys_arg` point to it.
    ///
    /// Note that this will "allocate" a block of memory on stack, which means
    /// the value of the stack pointer register will be implicitly modified.
    fn set_sysarg_data(
        &mut self,
        sys_arg: SysArgIndex,
        data: &[u8],
        justification: &'static str,
        append_null: bool,
    ) -> Result<()> {
        // Allocate space into the tracee's memory to host the new data.
        let tracee_ptr =
            self.alloc_mem_on_stack(data.len() as isize + if append_null { 1 } else { 0 })?;

        // Copy the new data into the previously allocated space.
        self.write_data(tracee_ptr as *mut Word, data, append_null)?;

        // Make this argument point to the new data.
        self.set(SysArg(sys_arg), tracee_ptr, justification);

        Ok(())
    }

    /// Copy the `data` to tracee's memory space by ptrace(PTRACE_POKEDATA) and
    /// ptrace(PTRACE_PEEKDATA). It transmits one word at a time, and the
    /// boundary case is carefully handled.
    fn write_data(&self, dest_tracee: *mut Word, data: &[u8], append_null: bool) -> Result<()> {
        //TODO implement belongs_to_heap_prealloc
        // if (belongs_to_heap_prealloc(tracee, dest_tracee))
        // return -EFAULT;

        // TODO: use process_vm_writev() to write data if process_vm feature was
        // supported.

        // if append null is required, we appen a b'\0' byte to the end of data.
        let mut buf = if append_null {
            let null_char_slice: &[u8] = &[b'\0'];
            data.chain(null_char_slice)
        } else {
            data.chain(&[] as &[u8])
        };
        let size = if append_null {
            // the +1 is for the `\0` byte that we will have manually
            data.len() + 1
        } else {
            data.len()
        };
        let word_size = mem::size_of::<Word>();
        let nb_trailing_bytes = (size % word_size) as isize;
        let nb_full_words = ((size - nb_trailing_bytes as usize) / word_size) as isize;

        // Copy one word by one word, except for the last one.
        for i in 0..nb_full_words {
            // The byteorder crate is used to read the [u8] slice as a [Word] slice.
            let word = buf.read_uint::<NativeEndian>(word_size).unwrap() as Word;
            let dest_addr = unsafe { dest_tracee.offset(i) as *mut c_void };

            unsafe { ptrace::write(self.get_pid(), dest_addr, word as *mut c_void)? };
        }

        // Copy the bytes in the last word carefully since we have to
        // overwrite only the relevant ones.
        let last_dest_addr = unsafe { dest_tracee.offset(nb_full_words) as *mut c_void };
        let existing_word = ptrace::read(self.get_pid(), last_dest_addr)? as Word;
        let mut bytes = convert_word_to_bytes(existing_word);

        // The trailing bytes are merged with the existing bytes. For example:
        // bytes = [0, 0, 0, 0, 0, 0, 119, 0] // the already existing bytes at the dest
        // addr trailing bytes = [164, 247, 274] // our trailing bytes
        // fusion = [164, 247, 274, 0, 0, 0, 119, 0] // the fusion of the two
        for byte in bytes.iter_mut().take(nb_trailing_bytes as usize) {
            *byte = buf.read_u8().unwrap();
        }

        let last_word = convert_bytes_to_word(bytes);
        // We can now safely write the final word.
        unsafe { ptrace::write(self.get_pid(), last_dest_addr, last_word as *mut c_void)? };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::fork_test;
    use crate::{
        register::{Current, Original, PtraceReader, SysArg1},
        utils::tests::get_test_rootfs_path,
    };
    use nix::unistd::execvp;
    use sc::nr::MKDIR;
    use std::ffi::CString;
    use std::path::PathBuf;

    #[test]
    fn test_write_set_sysarg_path_write_same_path() {
        let rootfs_path = get_test_rootfs_path();

        let test_path = "my/impossible/test/path";
        let test_path_2 = "my/second/impossible/test/path";

        fork_test(
            rootfs_path,
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
