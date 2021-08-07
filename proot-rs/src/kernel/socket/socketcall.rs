use crate::errors::Result;

pub fn enter() -> Result<()> {
    Ok(())

    //    word_t args_addr;
    //    word_t sock_addr_saved;
    //    word_t sock_addr;
    //    word_t size_addr;
    //    word_t size;
    //
    //    args_addr = peek_reg(tracee, CURRENT, SYSARG_2);
    //
    //    switch (peek_reg(tracee, CURRENT, SYSARG_1)) {
    //    case SYS_BIND:
    //    case SYS_CONNECT:
    //        /* Handle these cases below.  */
    //        status = 1;
    //        break;
    //
    //    case SYS_ACCEPT:
    //    case SYS_ACCEPT4:
    //        /* Nothing special to do if no sockaddr was specified.  */
    //        sock_addr = PEEK_WORD(SYSARG_ADDR(2), 0);
    //        if (sock_addr == 0) {
    //            status = 0;
    //            break;
    //        }
    //        special = true;
    //        /* Fall through.  */
    //    case SYS_GETSOCKNAME:
    //    case SYS_GETPEERNAME:
    //        /* Remember: PEEK_WORD puts -errno in status and breaks
    //         * if an error occured.  */
    //        size_addr =  PEEK_WORD(SYSARG_ADDR(3), 0);
    //        size = (int) PEEK_WORD(size_addr, special ? -EINVAL : 0);
    //
    //        /* See case PR_accept for explanation.  */
    //        poke_reg(tracee, SYSARG_6, size);
    //        status = 0;
    //        break;
    //
    //    default:
    //        status = 0;
    //        break;
    //    }
    //
    //    /* An error occured or there's nothing else to do.  */
    //    if (status <= 0)
    //        break;
    //
    //    /* Remember: PEEK_WORD puts -errno in status and breaks if an
    //     * error occured.  */
    //    sock_addr = PEEK_WORD(SYSARG_ADDR(2), 0);
    //    size      = PEEK_WORD(SYSARG_ADDR(3), 0);
    //
    //    sock_addr_saved = sock_addr;
    //    status = translate_socketcall_enter(tracee, &sock_addr, size);
    //    if (status <= 0)
    //        break;
    //
    //    /* These parameters are used/restored at the exit stage.  */
    //    poke_reg(tracee, SYSARG_5, sock_addr_saved);
    //    poke_reg(tracee, SYSARG_6, size);
    //
    //    /* Remember: POKE_WORD puts -errno in status and breaks if an
    //     * error occured.  */
    //    POKE_WORD(SYSARG_ADDR(2), sock_addr);
    //    POKE_WORD(SYSARG_ADDR(3), sizeof(struct sockaddr_un));
    //
    //    status = 0;
    //    break;
}

pub fn exit() -> Result<()> {
    //    word_t args_addr;
    //    word_t sock_addr;
    //    word_t size_addr;
    //    word_t max_size;
    //
    //    args_addr = peek_reg(tracee, ORIGINAL, SYSARG_2);
    //
    //    switch (peek_reg(tracee, ORIGINAL, SYSARG_1)) {
    //    case SYS_ACCEPT:
    //    case SYS_ACCEPT4:
    //        /* Nothing special to do if no sockaddr was specified.  */
    //        sock_addr = PEEK_WORD(SYSARG_ADDR(2));
    //        if (sock_addr == 0)
    //            return SyscallExitResult::None;
    //        /* Fall through.  */
    //    case SYS_GETSOCKNAME:
    //    case SYS_GETPEERNAME:
    //        /* Handle these cases below.  */
    //        status = 1;
    //        break;
    //
    //    case SYS_BIND:
    //    case SYS_CONNECT:
    //        /* Restore the initial parameters: this memory was
    //         * overwritten at the enter stage.  Remember: POKE_WORD
    //         * puts -errno in status and breaks if an error
    //         * occured.  */
    //        POKE_WORD(SYSARG_ADDR(2), peek_reg(tracee, MODIFIED, SYSARG_5));
    //        POKE_WORD(SYSARG_ADDR(3), peek_reg(tracee, MODIFIED, SYSARG_6));
    //
    //        status = 0;
    //        break;
    //
    //    default:
    //        status = 0;
    //        break;
    //    }
    //
    //    /* Error reported by the kernel or there's nothing else to do.  */
    //    if ((int) syscall_result < 0 || status == 0)
    //        return SyscallExitResult::None;
    //
    //    /* An error occured in SYS_BIND or SYS_CONNECT.  */
    //    if (status < 0)
    //        return SyscallExitResult::Value(status);
    //
    //    /* Remember: PEEK_WORD puts -errno in status and breaks if an
    //     * error occured.  */
    //    sock_addr = PEEK_WORD(SYSARG_ADDR(2));
    //    size_addr = PEEK_WORD(SYSARG_ADDR(3));
    //    max_size  = peek_reg(tracee, MODIFIED, SYSARG_6);
    //
    //    status = translate_socketcall_exit(tracee, sock_addr, size_addr,
    // max_size);    if (status < 0)
    //        return SyscallExitResult::Value(status);
    //
    // Don't overwrite the syscall result.
    Ok(())
}
