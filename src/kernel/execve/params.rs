use std::ffi::CString;
use std::path::PathBuf;

use libc::c_void;
use nix::{sys::ptrace, unistd::Pid};

use crate::errors::*;
use crate::filesystem::{FileSystem, Translator};
use crate::process::tracee::Tracee;
use crate::register::{PtraceWriter, Word};
use crate::utils::AsU8Slice;

/// Denotes the value of each argument in the `argv` argument list of the
/// `execve()` function. A argument is a C string, which ends with a '\0'.
///
/// These parameters may exist in the tracee's memory space, or they may be C
/// strings in the current process that have not yet been written into tracee.
#[derive(Debug)]
pub(super) enum Arg {
    /// Represents a C string in tracee's memory space, stored as a pointer.
    /// Note that we should never try to deref this pointer, since it is in the
    /// tracee's memory space.
    CStringInTracee(*const c_void),
    /// A C string exist in current memory space, which is usually will be
    /// written into tracee's memory space later.
    CStringInSelf(CString),
}

/// Parameters related to the `execve()` system call. This struct is mainly used
/// when loading and parsing executable files.
#[derive(Debug)]
pub(super) struct ExecveParameters {
    /// The original path to the executable file, from the first argument of
    /// `execve()`, or the interpreter path in the script file.
    ///
    /// It is the path on the guest side. Since it comes from user input, it is
    /// not necessarily a canonical path.
    pub raw_guest_path: PathBuf,
    /// It is the same as `raw_guest_path`, but it is a canonical path.
    ///
    /// This field should normally be read-only. To update it, see the
    /// `ExecveParameters::update_path()` function.
    pub canonical_guest_path: PathBuf,
    /// It is the same as `raw_guest_path`, but it is a canonical path, and is
    /// the path on the host side.
    ///
    /// This field should normally be read-only. To update it, see the
    /// `ExecveParameters::update_path()` function.
    pub host_path: PathBuf,
    /// Denotes the second argument of the `execve()` syscall.
    pub argv: Vec<Arg>,
}

impl ExecveParameters {
    /// Update the `canonical_guest_path` and `host_path` values based on the
    /// value of `raw_guest_path`.
    pub fn update_path(&mut self, fs: &FileSystem) -> Result<()> {
        let (canonical_guest_path, host_path) = fs.translate_path(&self.raw_guest_path, true)?;
        self.canonical_guest_path = canonical_guest_path;
        self.host_path = host_path;

        Ok(())
    }
}

/// Read arguments list (`argv`) from a tracee.
///
/// The `argv` is an array of pointers to a set of C strings, and end with a
/// null pointer. In this function, only the pointers of these C strings are
/// read. The contents of the C strings and the trailing null pointers will not
/// be read.
pub(super) fn read_argv(pid: Pid, addr: *const c_void) -> Result<Vec<Arg>> {
    let mut argv = vec![];
    let mut i = 0;

    loop {
        let word = ptrace::read(pid, unsafe { (addr as *mut Word).offset(i) } as _).with_context(
            || {
                format!(
                    "Failed to read argv from tracee. pid: {}, addr: 0x{:x?}, offset: {}",
                    pid, addr, i
                )
            },
        )?;
        if word == 0 {
            break;
        }
        argv.push(Arg::CStringInTracee(word as *const c_void));
        i += 1;
    }

    Ok(argv)
}

/// Write arguments list (`argv`) into a tracee's memory space.
///
/// This function will write a list of args into tracee, and then append a null
/// pointer, which is the opposite of `read_argv()`. In addition, for C strings
/// in the current process, the function will also copy them to tracee.
pub(super) fn write_argv(tracee: &mut Tracee, argv: &[Arg]) -> Result<*const c_void> {
    let mut new_argv = argv
        .iter()
        .map(|arg| -> _ {
            Ok(match arg {
                Arg::CStringInTracee(addr) => *addr,
                Arg::CStringInSelf(cstring) => tracee
                    .regs
                    .allocate_and_write(cstring.as_bytes_with_nul(), false)?
                    as _,
            })
        })
        .collect::<Result<Vec<*const c_void>>>()?;

    new_argv.push(std::ptr::null::<c_void>());

    Ok(tracee
        .regs
        .allocate_and_write(new_argv.as_u8_slice(), false)? as _)
}
