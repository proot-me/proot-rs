use syscalls::socket::get_sockorpeer_name;
use syscalls::syscall_exit::SyscallExitResult;

pub fn enter() {
    /* Nothing special to do if no sockaddr was specified.  */
    // if (peek_reg(tracee, ORIGINAL, SYSARG_2) == 0) {
    //     status = 0;
    //     break;
    // }
    // special = true;

    get_sockorpeer_name::enter();
}

pub fn exit() -> SyscallExitResult {
    /* Nothing special to do if no sockaddr was specified.  */
    // if (peek_reg(tracee, ORIGINAL, SYSARG_2) == 0)
    //      return SyscallExitResult::None;

    get_sockorpeer_name::exit()
}