
pub fn enter() {
    println!("accept");
    /* Nothing special to do if no sockaddr was specified.  */
    // if (peek_reg(tracee, ORIGINAL, SYSARG_2) == 0) {
    //     status = 0;
    //     break;
    // }
    // special = true;

    /* Remember: PEEK_WORD puts -errno in status and breaks if an
     * error occured.  */
    // size = (int) PEEK_WORD(peek_reg(tracee, ORIGINAL, SYSARG_3), special ? -EINVAL : 0);

    /* The "size" argument is both used as an input parameter
     * (max. size) and as an output parameter (actual size).  The
     * exit stage needs to know the max. size to not overwrite
     * anything, that's why it is copied in the 6th argument
     * (unused) before the kernel updates it.  */
    // poke_reg(tracee, SYSARG_6, size);

    // status = 0;
}