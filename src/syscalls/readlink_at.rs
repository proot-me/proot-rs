use syscalls::unlink_mkdir_at;

pub fn enter() {
    unlink_mkdir_at::enter();
}

pub fn exit() {
//    char referee[PATH_MAX];
//    char referer[PATH_MAX];
//    size_t old_size;
//    size_t new_size;
//    size_t max_size;
//    word_t input;
//    word_t output;
//
//    /* Error reported by the kernel.  */
//    if ((int) syscall_result < 0)
//        goto end;
//
//    old_size = syscall_result;
//
//    if (syscall_number == PR_readlink) {
//        output   = peek_reg(tracee, ORIGINAL, SYSARG_2);
//        max_size = peek_reg(tracee, ORIGINAL, SYSARG_3);
//        input    = peek_reg(tracee, MODIFIED, SYSARG_1);
//    }
//    else {
//        output   = peek_reg(tracee, ORIGINAL,  SYSARG_3);
//        max_size = peek_reg(tracee, ORIGINAL, SYSARG_4);
//        input    = peek_reg(tracee, MODIFIED, SYSARG_2);
//    }
//
//    if (max_size > PATH_MAX)
//        max_size = PATH_MAX;
//
//    if (max_size == 0) {
//        status = -EINVAL;
//        break;
//    }
//
//    /* The kernel does NOT put the NULL terminating byte for
//     * readlink(2).  */
//    status = read_data(tracee, referee, output, old_size);
//    if (status < 0)
//        break;
//    referee[old_size] = '\0';
//
//    /* Not optimal but safe (path is fully translated).  */
//    status = read_path(tracee, referer, input);
//    if (status < 0)
//        break;
//
//    if (status >= PATH_MAX) {
//        status = -ENAMETOOLONG;
//        break;
//    }
//
//    status = detranslate_path(tracee, referee, referer);
//    if (status < 0)
//        break;
//
//    /* The original path doesn't require any transformation, i.e
//     * it is a symetric binding.  */
//    if (status == 0)
//        goto end;
//
//    /* Overwrite the path.  Note: the output buffer might be
//     * initialized with zeros but it was updated with the kernel
//     * result, and then with the detranslated result.  This later
//     * might be shorter than the former, so it's safier to add a
//     * NULL terminating byte when possible.  This problem was
//     * exposed by IDA Demo 6.3.  */
//    if ((size_t) status < max_size) {
//        new_size = status - 1;
//        status = write_data(tracee, output, referee, status);
//    }
//    else {
//        new_size = max_size;
//        status = write_data(tracee, output, referee, max_size);
//    }
//    if (status < 0)
//        break;
//
//    /* The value of "status" is used to update the returned value
//     * in translate_syscall_exit().  */
//    status = new_size;
//    break;
}