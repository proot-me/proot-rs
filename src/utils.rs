
#[cfg(test)]
pub mod tests {
    use std::ptr::null_mut;
    use std::ops::Fn;
    use libc::pid_t;
    use nix::unistd::{getpid, fork, ForkResult};
    use nix::sys::signal::kill;
    use nix::sys::signal::Signal::SIGSTOP;
    use nix::sys::ptrace::ptrace;
    use nix::sys::ptrace::ptrace::PTRACE_TRACEME;
    use nix::sys::wait::{waitpid, __WALL};
    use nix::sys::wait::WaitStatus::*;
    use nix::sys::ptrace::ptrace::PTRACE_SYSCALL;
    use process::proot::InfoBag;
    use process::tracee::Tracee;
    use register::Registers;

    /// Allow tests to fork and deal with child processes without mixing them.
    fn test_in_subprocess<F: Fn()>(func: F) {
        let pid = fork();
        match pid {
            Ok(ForkResult::Child) => {
                func();
            }
            Ok(ForkResult::Parent { child }) => {
                assert_eq!(waitpid(child, None), Ok(Exited(child, 0)))
            }
            Err(_) => panic!("Error: fork"),
        }
    }

    /// Simulates PRoot by forking a parent and child processes.
    /// The child process will be traced on, and will execute its respective function (2nd arg).
    /// The parent process will wait and loop for events from the tracee (child process).
    /// It only stops when the parent function (1st arg) returns true.
    pub fn fork_test<FuncParent: Fn(pid_t, &Registers) -> bool, FuncChild: Fn()>(
        expected_exit_signal: i8,
        func_parent: FuncParent,
        func_child: FuncChild,
    ) {

        test_in_subprocess(|| {
            match fork().expect("fork in test") {
                ForkResult::Parent { child } => {
                    let info_bag = &mut InfoBag::new();
                    let tracee = Tracee::new(child);

                    // the parent will wait for the child's signal before calling set_ptrace_options
                    assert_eq!(
                        waitpid(-1, Some(__WALL)).expect("event loop waitpid"),
                        Stopped(child, SIGSTOP)
                    );
                    tracee.set_ptrace_options(info_bag);

                    restart(child);

                    // we loop until the parent function decides to stop
                    loop {
                        match waitpid(child, Some(__WALL)).expect("event loop waitpid") {
                            PtraceSyscall(pid) => {
                                assert_eq!(pid, child);
                                let regs = Registers::retrieve(pid).expect("fetch regs");

                                if func_parent(pid, &regs) {
                                    break;
                                }
                            }
                            Exited(_, _) => assert!(false),
                            Signaled(_, _, _) => assert!(false),
                            _ => {}
                        }
                        restart(child);
                    }

                    restart(child);
                    end(child, expected_exit_signal);
                }
                ForkResult::Child => {
                    ptrace(PTRACE_TRACEME, 0, null_mut(), null_mut()).expect("test ptrace traceme");
                    // we use a SIGSTOP to synchronise both processes
                    kill(getpid(), SIGSTOP).expect("test child sigstop");

                    func_child();
                }
            }
        });
    }

    /// Restarts a child process just once.
    fn restart(child: pid_t) {
        ptrace(PTRACE_SYSCALL, child, null_mut(), null_mut()).expect("exit tracee with exit stage");
    }

    /// Waits/restarts a child process until it stops.
    fn end(child: pid_t, expected_status: i8) {
        loop {
            match waitpid(child, Some(__WALL)).expect("waitpid") {
                Exited(pid, exit_status) => {
                    assert_eq!(pid, child);

                    // the tracee should have exited with an OK status (exit code 0)
                    assert_eq!(exit_status, expected_status);
                    break;
                }
                _ => {
                    // restarting the tracee
                    restart(child);
                }
            }
        }
    }
}
