use crate::errors::*;

use crate::process::tracee::Tracee;

use crate::register::{Current, SysArg, SysArg1, SysArg2};

pub fn enter(tracee: &mut Tracee) -> Result<()> {
    Ok(())
}

pub fn exit(tracee: &mut Tracee) -> Result<()> {
    //    char path[PATH_MAX];
    //    size_t new_size;
    //    size_t size;
    //    word_t output;
    //
    //    size = (size_t) peek_reg(tracee, ORIGINAL, SYSARG_2);
    //    if (size == 0) {
    //        status = -EINVAL;
    //        break;
    //    }
    //
    //    /* Ensure cwd still exists.  */
    //    status = translate_path(tracee, path, AT_FDCWD, ".", false);
    //    if (status < 0)
    //        break;
    //
    //    new_size = strlen(tracee->fs->cwd) + 1;
    //    if (size < new_size) {
    //        status = -ERANGE;
    //        break;
    //    }
    //
    //    /* Overwrite the path.  */
    //    output = peek_reg(tracee, ORIGINAL, SYSARG_1);
    //    status = write_data(tracee, output, tracee->fs->cwd, new_size);
    //    if (status < 0)
    //        break;
    //
    Ok(())
}
