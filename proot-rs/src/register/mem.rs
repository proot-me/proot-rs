use crate::errors::*;
use crate::register::{Current, Original, Registers, StackPointer, Word};
use std::usize::MAX as USIZE_MAX;

#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    target_arch = "x86_64"
))]
const RED_ZONE_SIZE: isize = 128;
#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    not(target_arch = "x86_64")
))]
const RED_ZONE_SIZE: isize = 0;

pub trait PtraceMemoryAllocator {
    fn alloc_mem_on_stack(&mut self, size: isize) -> Result<Word>;
}

impl PtraceMemoryAllocator for Registers {
    /// Allocate @size bytes in the @tracee's memory space.
    ///
    /// The register calling this method will have its stack pointer
    /// directly modified. The tracee is not modified now.
    /// The registers will have to be pushed for the updates to take place.
    ///
    /// This function should only be called in sysenter since the
    /// stack pointer is systematically restored at the end of
    /// sysexit (except for execve, but in this case the stack
    /// pointer should be handled with care since it is used by the
    /// process to retrieve argc, argv, envp, and auxv).
    ///
    /// `size` can be negative (no idea why; is it necessary?).
    ///
    /// Returns the address of the allocated memory in the @tracee's memory
    /// space, otherwise an error.
    fn alloc_mem_on_stack(&mut self, size: isize) -> Result<Word> {
        let original_stack_pointer = self.get(Original, StackPointer);
        let stack_pointer = self.get(Current, StackPointer);

        // Some ABIs specify an amount of bytes after the stack
        // pointer that shall not be used by anything but the compiler
        // (for optimization purpose).
        let corrected_size = match stack_pointer == original_stack_pointer {
            false => size,
            true => size + RED_ZONE_SIZE,
        };
        let overflow = corrected_size > 0 && stack_pointer <= corrected_size as Word;
        let underflow =
            corrected_size < 0 && stack_pointer >= (USIZE_MAX as Word) - (-corrected_size as Word);

        if overflow || underflow {
            //TODO: log warning
            // note(tracee, WARNING, INTERNAL, "integer under/overflow detected in %s",
            //     __FUNCTION__);
            return Err(Error::errno_with_msg(
                EFAULT,
                "when allocating memory, under/overflow detected",
            ));
        }

        // Remember the stack grows downward.
        let new_stack_pointer = match corrected_size > 0 {
            true => stack_pointer - (corrected_size as Word),
            false => stack_pointer + (-corrected_size as Word),
        };

        self.set(StackPointer, new_stack_pointer, "allocating memory");

        Ok(new_stack_pointer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::register::regs::RegisterSet;
    use crate::register::Registers;
    use nix::unistd::getpid;
    use std::mem;
    use std::usize::MAX;

    #[test]
    fn test_mem_alloc_normal() {
        let mut raw_regs: RegisterSet = unsafe { mem::zeroed() };
        let starting_stack_pointer = 100000;

        get_reg!(raw_regs, StackPointer) = starting_stack_pointer;

        let mut regs = Registers::from(getpid(), raw_regs);

        regs.save_current_regs(Original);

        let alloc_size = 7575;
        let new_stack_pointer = regs.alloc_mem_on_stack(alloc_size).unwrap();

        // Remember the stack grows downward.
        assert!(new_stack_pointer < starting_stack_pointer);
        assert_eq!(
            starting_stack_pointer - new_stack_pointer,
            alloc_size as Word + RED_ZONE_SIZE as Word
        );
    }

    #[test]
    fn test_mem_alloc_overflow() {
        let mut raw_regs: RegisterSet = unsafe { mem::zeroed() };
        let starting_stack_pointer = 120;

        get_reg!(raw_regs, StackPointer) = starting_stack_pointer;

        let mut regs = Registers::from(getpid(), raw_regs);

        regs.save_current_regs(Original);

        let alloc_size = 7575;
        let result = regs.alloc_mem_on_stack(alloc_size);

        assert_eq!(
            Err(Error::errno_with_msg(
                EFAULT,
                "when allocating memory, under/overflow detected",
            )),
            result
        );
    }

    #[test]
    fn test_mem_alloc_underflow() {
        let mut raw_regs: RegisterSet = unsafe { mem::zeroed() };
        let starting_stack_pointer = (MAX as Word) - 120;

        get_reg!(raw_regs, StackPointer) = starting_stack_pointer;

        let mut regs = Registers::from(getpid(), raw_regs);

        regs.save_current_regs(Original);

        let alloc_size = -7575;
        let result = regs.alloc_mem_on_stack(alloc_size);

        assert_eq!(
            Err(Error::errno_with_msg(
                EFAULT,
                "when allocating memory, under/overflow detected",
            )),
            result
        );
    }
}
