use syscalls::syscall_exit::SyscallExitResult;

#[cfg(all(target_os="linux", target_arch="x86_64"))]
pub fn exit() -> SyscallExitResult {
//    struct utsname utsname;
//    word_t address;
//    size_t size;
//
//    if (get_abi(tracee) != ABI_2)
//        return SyscallExitResult::None;
//
//    /* Error reported by the kernel.  */
//    if ((int) syscall_result < 0)
//        return SyscallExitResult::None;
//
//    address = peek_reg(tracee, ORIGINAL, SYSARG_1);
//
//    status = read_data(tracee, &utsname, address, sizeof(utsname));
//    if (status < 0)
//        return SyscallExitResult::Value(status);
//
//    /* Some 32-bit programs like package managers can be
//     * confused when the kernel reports "x86_64".  */
//    size = sizeof(utsname.machine);
//    strncpy(utsname.machine, "i686", size);
//    utsname.machine[size - 1] = '\0';
//
//    status = write_data(tracee, address, &utsname, sizeof(utsname));
//    if (status < 0)
//        return SyscallExitResult::Value(status);
//
    SyscallExitResult::Value(0)
}