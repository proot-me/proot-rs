use std::path::{Path, PathBuf};
use errors::Result;
use register::{Registers, SysArgIndex, PtraceMemoryAllocator};

pub trait PtraceWriter {
    fn sys_arg_path(&mut self, sys_arg: SysArgIndex, new_path: &Path) -> Result<()>;
}

impl PtraceWriter for Registers {
    fn sys_arg_path(&mut self, sys_arg: SysArgIndex, new_path: &Path) -> Result<()> {
        let tracee_ptr = self.alloc_mem(new_path.as_os_str().len() as isize)?;

        Ok(())
    }
}

/*

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
