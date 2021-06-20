#[cfg(test)]
pub mod tests {
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use std::{
        env,
        ffi::OsString,
        path::{Path, PathBuf},
    };

    use nix::sys::signal;
    use nix::sys::signal::kill;
    use nix::sys::signal::Signal::SIGSTOP;
    use nix::sys::wait;
    use nix::sys::wait::WaitStatus::*;
    use nix::sys::{ptrace, wait::WaitPidFlag};
    use nix::unistd;
    use nix::unistd::{fork, getpid, ForkResult, Pid};
    use signal::Signal;

    use crate::errors::*;
    use crate::filesystem::FileSystem;
    use crate::process::proot::InfoBag;
    use crate::process::proot::PRoot;
    use crate::process::tracee::Tracee;

    /// Allow tests to fork and deal with child processes without mixing them.
    ///
    /// Since each rust unit tests is executed in a different thread, we
    /// should fork a child process to test the proot, otherwise the
    /// calls to `waitpid(-1)` from different unit tests may affect each other
    fn test_in_subprocess<F: FnOnce()>(func: F) {
        let pid = unsafe { fork() };
        match pid {
            Ok(ForkResult::Child) => {
                // It seems that rust's unittest cannot capture the panic of the child process,
                // so we use `std::panic::catch_unwind()` to catch the exception and set the
                // exit code when panic occurs.
                match std::panic::catch_unwind(AssertUnwindSafe(|| func())) {
                    Ok(_) => std::process::exit(0),
                    Err(_) => std::process::exit(1),
                }
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
        FuncChild: FnOnce(),
    >(
        fs_root: P,
        expected_exit_signal: i8,
        mut func_parent: FuncParent,
        func_child: FuncChild,
    ) {
        test_in_subprocess(|| {
            match unsafe { fork() }.expect("fork in test") {
                ForkResult::Parent { child } => {
                    let mut info_bag = InfoBag::new();
                    let mut tracee = Tracee::new(
                        child,
                        Rc::new(RefCell::new(
                            FileSystem::with_root(fs_root.as_ref()).unwrap(),
                        )),
                    );

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

                    match std::panic::catch_unwind(AssertUnwindSafe(|| func_child())) {
                        Ok(_) => std::process::exit(0),
                        Err(_) => std::process::exit(1),
                    }
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

    /// This helper function is used to test `proot-rs`, which runs the main
    /// part of the `proo-rs` code and takes a function as tracee.
    /// `syscall-stop` hook function is also provided to check the status
    /// inside `proot-rs`.
    ///
    /// `func_syscall_hook` is a callback function which will be called when a
    /// `syscall-stop` arrives. The signature of the callback function is
    /// `Fn(&Tracee, bool, bool)`.
    /// The description of the parameters is as follows:
    /// - `tracee`: The tracee which received a `syscall-stop`, which is
    ///   immutable.
    /// - `is_sysenter`: `true` if it is a `syscall-enter-stop`, otherwise it is
    ///   a `syscall-exit-stop`.
    /// - `before_translation`: `true` if syscall translation has not yet
    ///   started, and `false` if the syscall translation has finished.
    ///
    /// `func_tracee` is a callback function which will run in the child
    /// process as a `tracee`. The child process will exit immediately after
    /// this `func_tracee` returns.
    pub fn test_with_proot<
        FuncSyscallHook: Fn(&Tracee, bool, bool) + 'static,
        FuncTracee: FnOnce(),
    >(
        func_syscall_hook: FuncSyscallHook,
        func_tracee: FuncTracee,
    ) {
        test_in_subprocess(|| {
            let func = || -> Result<()> {
                // setup FileSystem and PRoot
                let root_path = get_test_rootfs_path();
                let mut fs = FileSystem::with_root(root_path)?;
                fs.set_cwd("/")?;
                let mut proot: PRoot = PRoot::new();
                proot.func_syscall_hook = Some(Box::new(func_syscall_hook));
                // fork first sub process as tracee
                match unsafe { unistd::fork() }.context("Failed to fork() when starting process")? {
                    ForkResult::Parent { child } => {
                        proot.create_tracee(child, Rc::new(RefCell::new(fs)));
                        proot.init_pid = Some(child);
                    }
                    ForkResult::Child => {
                        let init_child_func = || -> Result<()> {
                            ptrace::traceme().context(
                                "Failed to execute ptrace::traceme() in a child process",
                            )?;
                            signal::kill(unistd::getpid(), Signal::SIGSTOP).context(
                                "Child process failed to synchronize with parent process",
                            )?;
                            match std::panic::catch_unwind(AssertUnwindSafe(|| func_tracee())) {
                                Ok(_) => std::process::exit(0),
                                Err(_) => std::process::exit(1),
                            }
                        };
                        if let Err(e) = init_child_func() {
                            error!("Failed to initialize the child process: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                // start up proot event loop
                proot.event_loop()?;

                assert_eq!(
                    proot.init_exit_code,
                    Some(0),
                    "tracee exited with a bad exit code: {:?}",
                    proot.init_exit_code
                );
                Ok(())
            };
            func().unwrap()
        })
    }

    /// Get the path to the new root fs for the unit test, which is specified by
    /// the environment variable `PROOT_TEST_ROOTFS`.
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

    #[test]
    #[should_panic]
    fn test_test_in_subprocess_assert_false() {
        test_in_subprocess(|| assert!(false));
    }

    #[test]
    fn test_test_in_subprocess_assert_true() {
        test_in_subprocess(|| assert!(true));
    }

    #[test]
    #[should_panic]
    fn test_fork_test_assert_false() {
        fork_test(
            get_test_rootfs_path(),
            0,
            |_tracee, _info_bag| {
                assert!(false);
                true
            },
            || {
                assert!(false);
            },
        );
    }

    #[test]
    fn test_fork_test_assert_true() {
        fork_test(
            get_test_rootfs_path(),
            0,
            |_tracee, _info_bag| {
                assert!(true);
                true
            },
            || {
                assert!(true);
            },
        );
    }

    #[test]
    #[should_panic]
    fn test_test_with_proot_assert_false() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| assert!(false),
            || assert!(false),
        )
    }

    #[test]
    fn test_test_with_proot_assert_true() {
        test_with_proot(
            |_tracee, _is_sysenter, _before_translation| assert!(true),
            || assert!(true),
        )
    }
}
