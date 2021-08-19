// Disable rust std lib
#![no_std]
// Disable rust main function
#![no_main]
#![feature(lang_items)]
#![feature(bindings_after_at)]
#![feature(llvm_asm)]
// Use `link_args` attribute to customize linking.
#![feature(link_args)]

#[allow(unused_attributes)]
// Only static links are used and prevent linking with the shared libraries.
#[link_args = "-no-pie"]
#[link_args = "-static"]
// Disable system startup files or libraries when linking. This means
// that the linker will not include files like `crt0.o` and some of the
// system standard libraries.
// See https://gcc.gnu.org/onlinedocs/gcc/Link-Options.html
//
// The `-nostdlib` flag is much like a combination of `-nostartfiles` and
// `-nodefaultlibs`.
//
// Since `_start` is defined in the system startup files, with this option
// we can use our own `_start` function to override the program entry point.
#[link_args = "-nostdlib"]
#[link_args = "-ffreestanding"]
#[cfg_attr(target_arch = "x86", link_args = "-mregparm=3")]
#[cfg_attr(target_arch = "x86", link_args = "-Wl,-Ttext=0xa0000000")]
#[cfg_attr(target_arch = "x86_64", link_args = "-Wl,-Ttext=0x600000000000")]
#[cfg_attr(target_arch = "arm", link_args = "-Wl,-Ttext=0x10000000")]
#[cfg_attr(target_arch = "aarch64", link_args = "-Wl,-Ttext=0x2000000000")]
extern "C" {}

// The compiler may emit a call to the `memset()` function even if there is
// no such call in our code. However, since we use `-nostdlib` or
// `-nodefaultlibs`, this means we will not link to libc, which provides the
// implementation of `memset()`.
//
// In this case, we will get an `undefined reference to \`memset'` error.
// Fortunately, the crate `rlibc` provides an unoptimized implementation of
// `memset()`.
//
// See `-nodefaultlibs` at https://gcc.gnu.org/onlinedocs/gcc/Link-Options.html
extern crate rlibc;

mod script;

use core::{fmt::Write, panic::PanicInfo};

use crate::script::*;

const O_RDONLY: usize = 00000000;
#[allow(dead_code)]
const AT_FDCWD: isize = -100;
const MAP_PRIVATE: usize = 0x02;
const MAP_FIXED: usize = 0x10;
const MAP_ANONYMOUS: usize = 0x20;

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
const MMAP_OFFSET_SHIFT: usize = 0;
#[cfg(any(target_arch = "arm"))]
const MMAP_OFFSET_SHIFT: usize = 12;

const PROT_READ: usize = 0x1;
const PROT_WRITE: usize = 0x2;
const PROT_EXEC: usize = 0x4;
const PROT_GROWSDOWN: usize = 0x01000000;

const AT_NULL: usize = 0;
const AT_PHDR: usize = 3;
const AT_PHENT: usize = 4;
const AT_PHNUM: usize = 5;
const AT_BASE: usize = 7;
const AT_ENTRY: usize = 9;
const AT_EXECFN: usize = 31;

const PR_SET_NAME: usize = 15;

macro_rules! branch {
    ($stack_pointer:expr, $entry_point:expr) => {
        #[cfg(target_arch = "x86_64")]
        llvm_asm!("
            // Restore initial stack pointer.
            movq $0, %rsp
            // Clear state flags.
            pushq $$0
            popfq
            // Clear rtld_fini.
            movq $$0, %rdx
            // Start the program.
            jmpq *%rax
        "
        : /* no output */
        : "irm" ($stack_pointer), "{ax}" ($entry_point)
        : "memory", "cc", "rsp", "rdx"
        : "volatile"
        );
        #[cfg(target_arch = "x86")]
        llvm_asm!("
            // Restore initial stack pointer
            movl $0, %esp
            // Clear state flags.
            pushl $$0
            popfl
            // Clear rtld_fini.
            movl $$0, %edx
            // Start the program.
            jmpl *%eax
        "
        : /* no output */
        : "irm" ($stack_pointer), "{ax}" ($entry_point)
        : "memory", "cc", "esp", "edx"
        : "volatile"
        );
        #[cfg(target_arch = "aarch64")]
        llvm_asm!("
            // Restore initial stack pointer
            mov sp, $0
            // Clear rtld_fini.
            mov x0, 0
            // Start the program.
            br $1
        "
        : /* no output */
        : "r" ($stack_pointer), "r" ($entry_point)
        : "memory", "x0"
        : "volatile"
        );
        #[cfg(target_arch = "arm")]
        llvm_asm!("
            // Restore initial stack pointer
            mov sp, $0
            // Clear rtld_fini.
            mov r0, $$0
            // Start the program.
            mov pc, $1
        "
        : /* no output */
        : "r" ($stack_pointer), "r" ($entry_point)
        : "memory", "r0"
        : "volatile"
        );

    }
}

/**
 * Interpret the load script pointed to by @cursor.
 */
#[no_mangle]
pub unsafe extern "C" fn _start(mut cursor: *const ()) {
    let mut traced = false;
    let mut reset_at_base = true;
    let mut at_base: Word = 0;
    let mut fd: Option<isize> = None;

    loop {
        // check if cursor is null
        // TODO: Check LoadStatement flag is vaild: Converting memory regions
        // directly to references to enum in rust is dangerous because invalid
        // tags can lead to undefined behaviors.
        let stmt: &LoadStatement = match (cursor as *const LoadStatement).as_ref() {
            Some(stmt) => stmt,
            None => panic!("Value of cursor is null"),
        };
        match stmt {
            st @ (LoadStatement::OpenNext(open) | LoadStatement::Open(open)) => {
                if let LoadStatement::OpenNext(_) = st {
                    // close last fd
                    assert!(sc::syscall!(CLOSE, fd.unwrap()) as isize >= 0);
                }
                // open file
                #[cfg(any(target_arch = "x86", target_arch = "arm", target_arch = "x86_64"))]
                let status = sc::syscall!(OPEN, open.string_address, O_RDONLY, 0) as isize;
                #[cfg(any(target_arch = "aarch64"))]
                let status =
                    sc::syscall!(OPENAT, AT_FDCWD, open.string_address, O_RDONLY, 0) as isize;
                assert!(status >= 0);
                fd = Some(status);
                reset_at_base = true
            }
            LoadStatement::MmapFile(mmap) => {
                // call mmap() with fd
                #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
                let status = sc::syscall!(
                    MMAP,
                    mmap.addr,
                    mmap.length,
                    mmap.prot,
                    MAP_PRIVATE | MAP_FIXED,
                    fd.unwrap(),
                    mmap.offset >> MMAP_OFFSET_SHIFT
                );
                #[cfg(any(target_arch = "arm", target_arch = "x86"))]
                let status = sc::syscall!(
                    MMAP2,
                    mmap.addr,
                    mmap.length,
                    mmap.prot,
                    MAP_PRIVATE | MAP_FIXED,
                    fd.unwrap(),
                    mmap.offset >> MMAP_OFFSET_SHIFT
                );
                assert_eq!(status, mmap.addr as _);
                // set the end of the space to 0, if needed.
                if mmap.clear_length != 0 {
                    let start = (mmap.addr + mmap.length - mmap.clear_length) as *mut u8;
                    for i in 0..mmap.clear_length {
                        *start.offset(i as isize) = 0u8;
                    }
                }
                // if value of AT_BASE need to be reset
                if reset_at_base {
                    at_base = mmap.addr;
                    reset_at_base = false;
                }
            }
            LoadStatement::MmapAnonymous(mmap) => {
                #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
                let status = sc::syscall!(
                    MMAP,
                    mmap.addr,
                    mmap.length,
                    mmap.prot,
                    MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS,
                    (-1isize) as usize,
                    0
                );
                #[cfg(any(target_arch = "arm", target_arch = "x86"))]
                let status = sc::syscall!(
                    MMAP2,
                    mmap.addr,
                    mmap.length,
                    mmap.prot,
                    MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS,
                    (-1isize) as usize,
                    0
                );

                assert!(status as isize >= 0);
            }
            LoadStatement::MakeStackExec(stack_exec) => {
                sc::syscall!(
                    MPROTECT,
                    stack_exec.start,
                    1,
                    PROT_READ | PROT_WRITE | PROT_EXEC | PROT_GROWSDOWN
                );
            }
            st @ (LoadStatement::StartTraced(start) | LoadStatement::Start(start)) => {
                if let LoadStatement::StartTraced(_) = st {
                    traced = true;
                }
                // close last fd
                assert!(sc::syscall!(CLOSE, fd.unwrap()) as isize >= 0);

                /* Right after execve, the stack content is as follow:
                 *
                 *   +------+--------+--------+--------+
                 *   | argc | argv[] | envp[] | auxv[] |
                 *   +------+--------+--------+--------+
                 */
                let mut cursor2: *mut Word = start.stack_pointer as _;
                let argc = *cursor2.offset(0);
                let at_execfn = *cursor2.offset(1);

                // skip argv[]
                cursor2 = cursor2.offset((argc + 1 + 1) as _);
                // the last element of argv should be a null pointer
                assert_eq!(*cursor2.offset(-1), 0);

                // skip envp[]
                while *cursor2 != 0 {
                    cursor2 = cursor2.offset(1)
                }
                cursor2 = cursor2.offset(1);

                // adjust auxv[]
                while *cursor2.offset(0) as usize != AT_NULL {
                    match *cursor2.offset(0) as usize {
                        AT_PHDR => *cursor2.offset(1) = start.at_phdr,
                        AT_PHENT => *cursor2.offset(1) = start.at_phent,
                        AT_PHNUM => *cursor2.offset(1) = start.at_phnum,
                        AT_ENTRY => *cursor2.offset(1) = start.at_entry,
                        AT_BASE => *cursor2.offset(1) = at_base,
                        AT_EXECFN => {
                            /* stmt->start.at_execfn can't be used for now since it is
                             * currently stored in a location that will be scratched
                             * by the process (below the final stack pointer).  */

                            *cursor2.offset(1) = at_execfn
                        }
                        _ => {}
                    }

                    cursor2 = cursor2.offset(2);
                }

                // get base name of executable path
                let get_basename = |string: *const u8| -> *const u8 {
                    let mut cursor = string;
                    while *cursor != 0 {
                        cursor = cursor.offset(1);
                    }
                    while *cursor != b'/' && cursor > string {
                        cursor = cursor.offset(-1);
                    }
                    if *cursor == b'/' {
                        cursor = cursor.offset(1);
                    }
                    cursor
                };
                let name = get_basename(start.at_execfn as _);
                sc::syscall!(PRCTL, PR_SET_NAME, name as usize, 0);

                // jump to new entry point
                if traced {
                    sc::syscall!(EXECVE, 1, start.stack_pointer, start.entry_point, 2, 3, 4);
                } else {
                    branch!(start.stack_pointer, start.entry_point);
                }
                unreachable!()
            }
        }
        // move cursor to next load statement
        cursor = (cursor as *const u8).offset(stmt.as_bytes().len() as _) as _;
    }
}

struct Stderr {}

impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bs = s.as_bytes();
        let mut count = 0;
        while count < bs.len() {
            unsafe {
                let status = sc::syscall!(WRITE, 2, bs.as_ptr().add(count), bs.len() - count);
                if (status as isize) < 0 {
                    return Err(core::fmt::Error);
                } else {
                    count += status;
                }
            }
        }
        Ok(())
    }
}

#[lang = "eh_personality"]
fn eh_personality() {}
#[panic_handler]
fn panic_handler(panic_info: &PanicInfo<'_>) -> ! {
    // If an error occurs, use the exit() system call to exit the program.
    let _ = write!(
        Stderr {},
        "An error occurred in loader-shim:\n{}\n",
        panic_info
    );
    unsafe {
        sc::syscall!(EXIT, 182);
    }
    unreachable!()
}

#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() -> () {
    loop {}
}
