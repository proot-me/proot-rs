
use nix::Result;

pub fn enter() -> Result<()> {
    Ok(())

    //                 dirfd = peek_reg(tracee, CURRENT, SYSARG_1);
    //
    //                 status = get_sysarg_path(tracee, path, SYSARG_2);
    //                 if (status < 0)
    //                    break;
    //
    //                 status = translate_path2(tracee, dirfd, path, SYSARG_2, REGULAR);
}