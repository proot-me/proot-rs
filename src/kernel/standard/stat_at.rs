use crate::errors::Result;

pub fn enter() -> Result<()> {
    Ok(())

    //                 dirfd = peek_reg(tracee, CURRENT, SYSARG_1);
    //
    //                 status = get_sysarg_path(tracee, path, SYSARG_2);
    //                 if (status < 0)
    //                 break;
    //
    //                 flags = (  syscall_number == PR_fchownat
    //                     || syscall_number == PR_name_to_handle_at)
    //                     ? peek_reg(tracee, CURRENT, SYSARG_5)
    //                     : peek_reg(tracee, CURRENT, SYSARG_4);
    //
    //                 if ((flags & AT_SYMLINK_NOFOLLOW) != 0)
    //                 status = translate_path2(tracee, dirfd, path, SYSARG_2, SYMLINK);
    //                 else
    //                 status = translate_path2(tracee, dirfd, path, SYSARG_2, REGULAR);
    //                 break;
}
