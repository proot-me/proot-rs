use kernel::standard::link_rename;
use kernel::exit::SyscallExitResult;
use errors::Result;

pub fn enter() -> Result<()> {
    Ok(())

    //                olddirfd = peek_reg(tracee, CURRENT, SYSARG_1);
    //                newdirfd = peek_reg(tracee, CURRENT, SYSARG_3);
    //
    //                status = get_sysarg_path(tracee, oldpath, SYSARG_2);
    //                if (status < 0)
    //                break;
    //
    //                status = get_sysarg_path(tracee, newpath, SYSARG_4);
    //                if (status < 0)
    //                break;
    //
    //                status = translate_path2(tracee, olddirfd, oldpath, SYSARG_2, SYMLINK);
    //                if (status < 0)
    //                break;
    //
    //                status = translate_path2(tracee, newdirfd, newpath, SYSARG_4, SYMLINK);
}

pub fn exit() -> SyscallExitResult {
    link_rename::exit()
}
