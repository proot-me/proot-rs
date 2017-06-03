
pub fn enter() {
    println!("linkat");

    //                 olddirfd = peek_reg(tracee, CURRENT, SYSARG_1);
    //                 newdirfd = peek_reg(tracee, CURRENT, SYSARG_3);
    //                 flags    = peek_reg(tracee, CURRENT, SYSARG_5);
    //
    //                 status = get_sysarg_path(tracee, oldpath, SYSARG_2);
    //                 if (status < 0)
    //                 break;
    //
    //                 status = get_sysarg_path(tracee, newpath, SYSARG_4);
    //                 if (status < 0)
    //                  break;
    //
    //                 if ((flags & AT_SYMLINK_FOLLOW) != 0)
    //                     status = translate_path2(tracee, olddirfd, oldpath, SYSARG_2, REGULAR);
    //                 else
    //                    status = translate_path2(tracee, olddirfd, oldpath, SYSARG_2, SYMLINK);
    //                 if (status < 0)
    //                 break;
    //                status = translate_path2(tracee, newdirfd, newpath, SYSARG_4, SYMLINK);
}