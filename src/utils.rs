#[cfg(test)]
pub mod tests {
    use crate::filesystem::FileSystem;
    use crate::process::proot::InfoBag;
    use crate::process::tracee::Tracee;
    use nix::sys::signal::kill;
    use nix::sys::signal::Signal::SIGSTOP;
    use nix::sys::wait;
    use nix::sys::wait::WaitStatus::*;
    use nix::sys::{ptrace, wait::WaitPidFlag};
    use nix::unistd::{fork, getpid, ForkResult, Pid};
    use std::{
        env,
        ffi::OsString,
        path::{Path, PathBuf},
    };

    /// Allow tests to fork and deal with child processes without mixing them.
    fn test_in_subprocess<F: FnMut()>(mut func: F) {
        let pid = unsafe { fork() };
        match pid {
            Ok(ForkResult::Child) => {
                func();
            }
            Ok(ForkResult::Parent { child }) => {
                assert_eq!(wait::waitpid(child, None), Ok(Exited(child, 0)))
            }
            Err(_) => panic!("Error: fork"),
        }
    }

    /// Simulates PRoot by forking a parent and child processes.
    /// The child process will be traced on, and will execute its respective
    /// function (2nd arg). The parent process will wait and loop for events
    /// from the tracee (child process). It only stops when the parent
    /// function (1st arg) returns true.
    pub fn fork_test<
        P: AsRef<Path>,
        FuncParent: FnMut(&mut Tracee, &mut InfoBag) -> bool,
        FuncChild: FnMut(),
    >(
        fs_root: P,
        expected_exit_signal: i8,
        mut func_parent: FuncParent,
        mut func_child: FuncChild,
    ) {
        test_in_subprocess(|| {
            match unsafe { fork() }.expect("fork in test") {
                ForkResult::Parent { child } => {
                    let mut info_bag = InfoBag::new();
                    let mut tracee =
                        Tracee::new(child, FileSystem::with_root(fs_root.as_ref()).unwrap());

                    // the parent will wait for the child's signal before calling set_ptrace_options
                    assert_eq!(
                        wait::waitpid(child, Some(WaitPidFlag::__WALL))
                            .expect("event loop waitpid"),
                        Stopped(child, SIGSTOP)
                    );
                    tracee
                        .check_and_set_ptrace_options(&mut info_bag)
                        .expect("error when set ptrace options");

                    restart(child);

                    // we loop until the parent function decides to stop
                    loop {
                        match wait::waitpid(child, Some(WaitPidFlag::__WALL))
                            .expect("event loop waitpid")
                        {
                            PtraceSyscall(pid) => {
                                assert_eq!(pid, child);
                                tracee.regs.fetch_regs().expect("fetch regs");

                                if func_parent(&mut tracee, &mut info_bag) {
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
                    ptrace::traceme().expect("test ptrace traceme");
                    // we use a SIGSTOP to synchronise both processes
                    kill(getpid(), SIGSTOP).expect("test child sigstop");

                    func_child();
                }
            }
        });
    }

    /// Restarts a child process just once.
    fn restart(child: Pid) {
        ptrace::syscall(child, None).expect("exit tracee with exit stage");
    }

    /// Waits/restarts a child process until it stops.
    fn end(child: Pid, expected_status: i8) {
        loop {
            match wait::waitpid(child, Some(WaitPidFlag::__WALL)).expect("waitpid") {
                Exited(pid, exit_status) => {
                    assert_eq!(pid, child);

                    // the tracee should have exited with the expected status
                    assert_eq!(exit_status as i8, expected_status);
                    break;
                }
                _ => {
                    // restarting the tracee
                    restart(child);
                }
            }
        }
    }

    pub fn get_test_rootfs_path() -> PathBuf {
        if let Some(val) = env::var_os("PROOT_TEST_ROOTFS") {
            if !val.is_empty() {
                let path = Path::new(val.as_os_str());
                if path.exists() && path.metadata().unwrap().is_dir() {
                    return std::fs::canonicalize(path).unwrap();
                }
                panic!("The guest rootfs path is invalid: {:?}", val);
            }
        }
        panic!("Unknown guest rootfs path: Please set the environment variable PROOT_TEST_ROOTFS to the path of the guest rootfs")
    }
}
