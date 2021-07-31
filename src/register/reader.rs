use crate::errors::*;
use crate::register::{Current, Registers, SysArg, SysArgIndex, Word};
use libc::{c_void, PATH_MAX};
use nix::sys::ptrace;
use nix::unistd::Pid;
use std::mem::{size_of, transmute};
use std::path::PathBuf;

#[cfg(target_pointer_width = "32")]
#[inline]
pub fn convert_word_to_bytes(value_to_convert: Word) -> [u8; 4] {
    unsafe { transmute(value_to_convert) }
}

#[cfg(target_pointer_width = "64")]
#[inline]
pub fn convert_word_to_bytes(value_to_convert: Word) -> [u8; 8] {
    unsafe { transmute(value_to_convert) }
}

pub trait PtraceReader {
    fn get_sysarg_path(&self, sys_arg: SysArgIndex) -> Result<PathBuf>;
}

impl PtraceReader for Registers {
    /// Retrieves a path from one of the syscall's arguments.
    ///
    /// Returns `Ok(path)` with path being a valid path if successful,
    /// `Ok(PathBuf::new())` if the syscall argument is null, or an error.
    #[inline]
    fn get_sysarg_path(&self, sys_arg: SysArgIndex) -> Result<PathBuf> {
        let src_sysarg = self.get(Current, SysArg(sys_arg)) as *mut Word;

        if src_sysarg.is_null() {
            trace!("{:?}({:x?}) => null", sys_arg, src_sysarg);
            // Check if the parameter is not NULL. Technically we should
            // not return an error for this special value since it is
            // allowed for some kernel, utimensat(2) for instance.
            Ok(PathBuf::new())
        } else {
            // Get the path from the tracee's memory space.
            let path = read_path(self.get_pid(), src_sysarg);
            match &path {
                Ok(path) => trace!("{:?}({:x?}) => {:?}", sys_arg, src_sysarg, path),
                Err(error) => trace!(
                    "{:?}({:x?}) => {:?}",
                    sys_arg,
                    src_sysarg,
                    error.get_errno()
                ),
            }
            path
        }
    }
}

/// Intermediary function that retrieves bytes from the tracee's memory space
/// and collects them into a NON null-terminated CString.
///
/// It also checks that the number of bytes isn't too long.
#[inline]
fn read_path(pid: Pid, src_path: *mut Word) -> Result<PathBuf> {
    let bytes = read_string(pid, src_path, PATH_MAX as usize)?;

    if bytes.len() >= PATH_MAX as usize {
        return Err(Error::errno_with_msg(
            ENAMETOOLONG,
            format!(
                "Error when reading sys arg path, path length {} exceed PATH_MAX {}",
                bytes.len(),
                PATH_MAX
            ),
        ));
    }

    Ok(PathBuf::from(unsafe { String::from_utf8_unchecked(bytes) }))
}

/// Reads a string from the memory space of a tracee.
///
/// It uses `ptrace(PEEK_DATA)` to read it word by word
/// (1 word = 1 c_ulong = 1 u32 or 1 u64 = 4 or 8 u8 = 4 or 8 char).
/// The copy stops when a null character `\0` is encountered (which is not
/// added), The bytes contained at the string's address are returned as a Vector
/// of u8.
///
/// * `pid` is the pid of the tracee.
/// * `src_string` is the address of the string in tracee's memory space
///   (obtained for instance with `get_reg`).
/// * `max_size` is the maximum number of bytes copied from memory.
fn read_string(pid: Pid, src_string: *mut Word, max_size: usize) -> Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::with_capacity(max_size);

    //TODO: belongs_to_heap_prealloc
    // if (belongs_to_heap_prealloc(tracee, dest_tracee))
    //	return -EFAULT;

    //TODO: implement HAVE_PROCESS_VM

    let word_size = size_of::<Word>();
    let nb_trailing_bytes = (max_size % word_size) as isize;
    let nb_full_words = ((max_size - nb_trailing_bytes as usize) / word_size) as isize;

    // Copy one word by one word, except for the last one.
    for i in 0..nb_full_words {
        let src_addr = unsafe { src_string.offset(i) as *mut c_void };

        // ptrace returns a c_long/Word that we will interpret as an 8-letters word
        let word = ptrace::read(pid, src_addr)? as Word;
        let letters = convert_word_to_bytes(word);

        for &letter in &letters {
            // Stop once an end-of-string is detected.
            if letter as char == '\0' {
                // bytes.push(letter); // we do not add the null byte to the path
                bytes.shrink_to_fit();

                return Ok(bytes);
            }
            bytes.push(letter);
        }
    }

    //todo: add trailing bytes processing (when necessary, need an example where
    // it's actually used)
    unimplemented!("trailing bytes not supported!")

    /*

    /* Copy the bytes from the last word carefully since we have
     * to not overwrite the bytes lying beyond @dest_tracer. */

    word = ptrace(PTRACE_PEEKDATA, tracee->pid, src + i, NULL);
    if (errno != 0)
        return -EFAULT;

    dest_word = (uint8_t *)&dest[i];
    src_word  = (uint8_t *)&word;

    for (j = 0; j < nb_trailing_bytes; j++) {
        dest_word[j] = src_word[j];
        if (src_word[j] == '\0')
            break;
    }

    return i * sizeof(word_t) + j + 1;
    */
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::register::regs::RegisterSet;
    use crate::register::*;
    use crate::utils::tests::{fork_test, get_test_rootfs_path};
    use nix::unistd::{execvp, getpid};
    use sc::nr::MKDIR;
    use std::ffi::CString;
    use std::mem;

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn test_reader_convert_word_to_bytes() {
        let number: Word = 'h' as u64
            + 'e' as u64 * 256
            + 'l' as u64 * 256 * 256
            + 'l' as u64 * 256 * 256 * 256
            + 'o' as u64 * 256 * 256 * 256 * 256;
        let bytes = convert_word_to_bytes(number);

        assert_eq!(bytes, [b'h', b'e', b'l', b'l', b'o', 0, 0, 0,]);
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn test_reader_convert_word_to_bytes() {
        let number: Word =
            'h' as u64 + 'e' as u64 * 256 + 'l' as u64 * 256 * 256 + 'o' as u64 * 256 * 256 * 256;
        let bytes = convert_word_to_bytes(number);

        assert_eq!(bytes, ['h' as u8, 'e' as u8, 'l' as u8, 'o' as u8]);
    }

    #[test]
    fn test_reader_get_sysarg_path_return_empty_if_given_null_src_() {
        let raw_regs: RegisterSet = unsafe { mem::zeroed() };
        let regs = Registers::from(getpid(), raw_regs);
        let args = [SysArg1, SysArg2, SysArg3, SysArg4, SysArg5, SysArg6];

        for arg in args.iter() {
            assert_eq!(regs.get_sysarg_path(*arg).unwrap().to_str().unwrap(), "");
        }
    }

    #[test]
    /// Tests that `get_sysarg_path`, `read_path` and `read_string` all work on
    /// a simple syscall, and succeeds in reading a syscall's path argument.
    ///
    /// The test is a success if the MKDIR syscall is detected (with its
    /// corresponding signum), and if the first argument of the syscall
    /// correspond to the path given to the initial command.
    fn test_reader_get_sysarg_path_for_mkdir_test() {
        let rootfs_path = get_test_rootfs_path();
        let test_path = "my/impossible/test/path";

        fork_test(
            rootfs_path,
            // expecting an error (because the path doesn't exit)
            1,
            // parent
            |tracee, _| {
                if tracee.regs.get_sys_num(Current) == MKDIR {
                    let dir_path = tracee.regs.get_sysarg_path(SysArg1).unwrap();

                    // we're checking that the string read in the tracee's memory
                    // corresponds to what has been given to the execve command
                    assert_eq!(dir_path, PathBuf::from(test_path));

                    // we can stop here
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
