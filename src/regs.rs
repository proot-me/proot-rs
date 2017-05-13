use std::ptr::null_mut;
use libc::{pid_t, c_void};
use nix::sys::ptrace::ptrace;
use nix::sys::ptrace::ptrace::PTRACE_GETREGS;

macro_rules! __item {
    ($i:item) => ($i)
}

/// Helper that transforms a Rust structure into a C structure
/// by adding `#[repr(C)]` on top of it, and making it copyable and cloneable.
/// The unroll part is there to gain (code) space by grouping fields
/// that have the same type. For instance:
/// pub [a, b, c] : u64
/// will be translated to:
/// pub a : u64,
/// pub b : u64,
/// pub c : u64,
macro_rules! unroll_and_structure {
    ($($(#[$attr:meta])*
        pub struct $i:ident {
            $(pub [ $( $field:ident ),* ]: $tt:ty),*
        }
    )*) => ($(
        __item! {
            #[repr(C)]
            $(#[$attr])*
            pub struct $i { $( $(pub $field: $tt),*),* }
        }
        impl Copy for $i {}
        impl Clone for $i {
            fn clone(&self) -> $i { *self }
        }
    )*)
}

/// Same as above, but with the new() method to instantiate it.
/// Though it will only work for structures that do not have array fields.
macro_rules! unroll_and_structure_and_impl {
    ($($(#[$attr:meta])*
        pub struct $i:ident {
            $(pub [ $( $field:ident ),* ]: $tt:ty),*
        }
    )*) => ($(
        __item! {
            #[repr(C)]
            $(#[$attr])*
            pub struct $i { $( $(pub $field: $tt),*),* }
        }
        impl $i {
            pub fn new() -> $i {
                $i {
                    $( $( $field: 0),*),*
                }
            }
        }
        impl Copy for $i {}
        impl Clone for $i {
            fn clone(&self) -> $i { *self }
        }
    )*)
}

/// Used to get the byte offset of a field in a structure.
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
}

/// The following structures are there to get the offset of the register's fields
/// (syscall number, arg1, arg2, etc...) in the current architecture's structures.
///
/// Reminder: the order in which the fields are declared is paramount.
/// `[repr(C)]` ensures that it stays the same when transformed in a C struct.

#[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
mod regs_structs {
    unroll_and_structure_and_impl! {
        #[derive(Debug)]
        pub struct user_regs_struct {
            pub [r15, r14, r13, r12, rbp, rbx, r11, r10, r9, r8, rax, rcx, rdx, rsi, rdi, orig_rax,
            rip, cs, eflags, rsp, ss, fs_base, gs_base, ds, es, fs, gs]: u64
        }
    }
    unroll_and_structure! {
        pub struct user_fpregs_struct {
            pub [cwd, swd, ftw, fop]: u16, pub [rip, rdp]: u64, pub [mxcsr, mxcr_mask]: u32,
            pub [st_space]: [u32; 32], pub [xmm_space]: [u32; 64], pub [padding]: [u32; 24]
        }
        pub struct user {
            pub [regs]: user_regs_struct, pub [u_fpvalid]: i32, pub [i387]: user_fpregs_struct,
            pub [u_tsize, u_dsize, u_ssize, start_code, start_stack]: u64, pub [signal]: i64,
            pub [reserved]: i32, pub [u_ar0]: *mut user_fpregs_struct,
            pub [u_fpstate]: *mut user_fpregs_struct,
            pub [magic]: u64, pub [u_comm]: [i8; 32], pub [u_debugreg] : [u64; 8]
        }
    }
}

#[cfg(all(target_os = "linux", any(target_arch = "x86")))]
mod regs_structs {
    unroll_and_structure! {

    }
}

#[cfg(all(target_os = "linux", any(target_arch = "arm")))]
mod regs_structs {

}

pub enum Reg {
    SysArgNum = 0,
    SysArg1,
    SysArg2,
    SysArg3,
    SysArg4,
    SysArg5,
    SysArg6,
    SysArgResult,
    StackPointer,
    InstrPointer,
    RtldFini,
    StateFlags,
    UserArg1,
}
pub const REG_SIZE: usize = 13;

macro_rules! user_regs_offset {
    ($reg_name:ident) => (offset_of!(user, regs) + offset_of!(user_regs_struct, $reg_name) );
}


/// Specify the ABI registers (syscall argument passing, stack pointer).
/// See sysdeps/unix/sysv/linux/${ARCH}/syscall.S from the GNU C Library.

#[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
pub mod regs_offset {
    use regs::REG_SIZE;
    use regs::regs_structs::*;

    pub unsafe fn get_regs_offsets() -> [usize; REG_SIZE] {
        [
            user_regs_offset!(orig_rax),    // SysArgNum
            user_regs_offset!(rdi),         // SysArg1
            user_regs_offset!(rsi),         // SysArg2
            user_regs_offset!(rdx),         // SysArg3
            user_regs_offset!(r10),         // SysArg4
            user_regs_offset!(r8),          // SysArg5
            user_regs_offset!(r9),          // SysArg6
            user_regs_offset!(rax),         // SysArgResult
            user_regs_offset!(rsp),         // StackPointer
            user_regs_offset!(rip),         // InstrPointer
            user_regs_offset!(rdx),         // RtldFini
            user_regs_offset!(eflags),      // StateFlags
            user_regs_offset!(rdi),         // UserArg1
        ]
    }
}

use self::regs_structs::user_regs_struct;

/// Copy all @tracee's general purpose registers into a dedicated cache.
pub unsafe fn fetch_regs(pid: pid_t) {
    let mut regs: user_regs_struct = user_regs_struct::new();
    let p_regs: *mut c_void = &mut regs as *mut _ as *mut c_void;

    ptrace(PTRACE_GETREGS, pid, null_mut(), p_regs).expect("get regs");
    println!("{:?}", regs);

    //TODO: convert regs into a more usable Rust structure, and return it
}