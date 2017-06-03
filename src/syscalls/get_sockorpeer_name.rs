
pub fn enter() {
//    /* Remember: PEEK_WORD puts -errno in status and breaks if an
//     * error occured.  */
//    // size = (int) PEEK_WORD(peek_reg(tracee, ORIGINAL, SYSARG_3), special ? -EINVAL : 0);
//
//    /* The "size" argument is both used as an input parameter
//     * (max. size) and as an output parameter (actual size).  The
//     * exit stage needs to know the max. size to not overwrite
//     * anything, that's why it is copied in the 6th argument
//     * (unused) before the kernel updates it.  */
//    // poke_reg(tracee, SYSARG_6, size);
//
//    // status = 0;
}

pub fn exit() {
//    word_t sock_addr;
//    word_t size_addr;
//    word_t max_size;
//
//    /* Error reported by the kernel.  */
//    if ((int) syscall_result < 0)
//        goto end;
//
//    sock_addr = peek_reg(tracee, ORIGINAL, SYSARG_2);
//    size_addr = peek_reg(tracee, MODIFIED, SYSARG_3);
//    max_size  = peek_reg(tracee, MODIFIED, SYSARG_6);
//
//    status = translate_socketcall_exit(tracee, sock_addr, size_addr, max_size);
//    if (status < 0)
//        break;
//
//    /* Don't overwrite the syscall result.  */
//    goto end;
}