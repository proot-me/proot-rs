use nix::Result;

pub fn enter() -> Result<()> {
    Ok(())

    //                status = get_sysarg_path(tracee, path, SYSARG_1);
    //                if (status < 0)
    //                break;
    //
    //                /* The following check covers only 90% of the cases. */
    //                if (path[0] == '/' || path[0] == '.') {
    //                    status = translate_path2(tracee, AT_FDCWD, path, SYSARG_1, REGULAR);
    //                    if (status < 0)
    //                    break;
    //                }
    //
    //                status = translate_sysarg(tracee, SYSARG_2, REGULAR);
}
