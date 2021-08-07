use crate::errors::Result;

use crate::kernel::socket::get_sockorpeer_name;

pub fn enter() -> Result<()> {
    get_sockorpeer_name::enter()

    /* Nothing special to do if no sockaddr was specified. */
    // if (peek_reg(tracee, ORIGINAL, SYSARG_2) == 0) {
    //     status = 0;
    //     break;
    // }
    // special = true;
}

pub fn exit() -> Result<()> {
    /* Nothing special to do if no sockaddr was specified. */
    // if (peek_reg(tracee, ORIGINAL, SYSARG_2) == 0)
    //      return SyscallExitResult::None;

    get_sockorpeer_name::exit()
}
