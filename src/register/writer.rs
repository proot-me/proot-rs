use std::path::{Path, PathBuf};
use errors::Result;
use register::{Registers, SysArgIndex};

pub trait PtraceWriter {
    fn sys_arg_path(&self, sys_arg: SysArgIndex, new_path: &Path) -> Result<()>;
}

impl PtraceWriter for Registers {
    fn sys_arg_path(&self, sys_arg: SysArgIndex, new_path: &Path) -> Result<()> {
        Ok(())
    }
}

/*

/**
 * Copy @size bytes of the data pointed to by @tracer_ptr into a
 * @tracee's memory block and make the @reg argument of the current
 * syscall points to this new block.  This function returns -errno if
 * an error occured, otherwise 0.
 */
static int set_sysarg_data(Tracee *tracee, const void *tracer_ptr, word_t size, Reg reg)
{
	word_t tracee_ptr;
	int status;

	/* Allocate space into the tracee's memory to host the new data. */
	tracee_ptr = alloc_mem(tracee, size);
	if (tracee_ptr == 0)
		return -EFAULT;

	/* Copy the new data into the previously allocated space. */
	status = write_data(tracee, tracee_ptr, tracer_ptr, size);
	if (status < 0)
		return status;

	/* Make this argument point to the new data. */
	poke_reg(tracee, reg, tracee_ptr);

	return 0;
}

*/
