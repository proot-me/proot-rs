use std::path::PathBuf;

use nix::sys::ptrace::{self, Options};
use nix::sys::signal::Signal;
use nix::unistd::Pid;

use crate::errors::Result;
use crate::errors::{Error, WithContext};
use crate::filesystem::FileSystem;
use crate::process::proot::InfoBag;
use crate::register::Registers;

#[derive(Debug, PartialEq)]
pub enum TraceeStatus {
    /// Enter syscall
    SysEnter,
    /// Exit syscall with no error
    SysExit,
    /// Exit syscall with error
    Error(Error),
}

impl TraceeStatus {
    pub fn is_err(&self) -> bool {
        matches!(*self, TraceeStatus::Error(_))
    }

    pub fn is_ok(&self) -> bool {
        !self.is_err()
    }

    pub fn get_errno(&self) -> i32 {
        match self {
            TraceeStatus::Error(err) => err.get_errno() as i32,
            _ => 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TraceeRestartMethod {
    /// Restart the tracee, without going through the exit stage
    WithoutExitStage, // PTRACE_CONT
    /// Restart the tracee, with the exit stage
    WithExitStage, // PTRACE_SYSCALL,
    /// Do not restart the tracee
    None,
}

#[derive(Debug)]
pub struct Tracee {
    /// Process identifier.
    pub pid: Pid,
    /// Whether the tracee is in the enter or exit stage.
    pub status: TraceeStatus,
    /// The ptrace's restart method depends on the status (enter or exit) and
    /// seccomp on/off.
    pub restart_how: TraceeRestartMethod,
    /// Contains the bindings and functions used for path translation.
    pub fs: FileSystem,
    /// Cached version of the process' general purpose registers.
    pub regs: Registers,
    /// State of the seccomp acceleration for this tracee.
    pub seccomp: bool,
    /// Ensure the sysexit stage is always hit under seccomp.
    pub sysexit_pending: bool,
    /// Path to the executable, à la /proc/self/exe. Used in `execve` enter.
    pub new_exe: Option<PathBuf>,
    /// Path to the executable, à la /proc/self/exe. Used in `execve` exit.
    pub exe: Option<PathBuf>,
}

impl Tracee {
    pub fn new(pid: Pid, fs: FileSystem) -> Tracee {
        Tracee {
            pid: pid,
            status: TraceeStatus::SysEnter, // it always starts by the enter stage
            restart_how: TraceeRestartMethod::None,
            fs: fs,
            regs: Registers::new(pid),
            seccomp: false,
            sysexit_pending: false,
            new_exe: None,
            exe: None,
        }
    }

    #[inline]
    pub fn reset_restart_how(&mut self) {
        // the restart method might already have been set elsewhere
        if self.restart_how == TraceeRestartMethod::None {
            // When seccomp is enabled, all events are restarted in
            // non-stop mode, but this default choice could be overwritten
            // later if necessary.  The check against "sysexit_pending"
            // ensures WithExitStage/PTRACE_SYSCALL (used to hit the exit stage under
            // seccomp) is not cleared due to an event that would happen
            // before the exit stage, eg. PTRACE_EVENT_EXEC for the exit
            // stage of kernel.execve(2).
            if self.seccomp && !self.sysexit_pending {
                self.restart_how = TraceeRestartMethod::WithoutExitStage;
            } else {
                self.restart_how = TraceeRestartMethod::WithExitStage;
            }
        }
    }

    #[inline]
    pub fn restart<T: Into<Option<Signal>>>(&mut self, sig: T) {
        match self.restart_how {
            TraceeRestartMethod::WithoutExitStage => {
                ptrace::cont(self.pid, sig).expect("exit tracee without exit stage");
            }
            TraceeRestartMethod::WithExitStage => {
                ptrace::syscall(self.pid, sig).expect("exit tracee with exit stage");
            }
            TraceeRestartMethod::None => {}
        };

        // the restart method is reinitialised here
        self.restart_how = TraceeRestartMethod::None;
    }

    /// Distinguish some events from others and
    /// automatically trace each new process with
    /// the same options.
    ///
    /// Note that only the first bare SIGTRAP is
    /// related to the tracing loop, others SIGTRAP
    /// carry tracing information because of
    /// TRACE*FORK/CLONE/EXEC.
    pub fn check_and_set_ptrace_options(&self, info_bag: &mut InfoBag) -> Result<()> {
        if info_bag.options_already_set {
            return Ok(());
        } else {
            info_bag.options_already_set = true;
        }

        let default_options = Options::PTRACE_O_TRACESYSGOOD
            | Options::PTRACE_O_TRACEFORK
            | Options::PTRACE_O_TRACEVFORK
            | Options::PTRACE_O_TRACEVFORKDONE
            | Options::PTRACE_O_TRACEEXEC
            | Options::PTRACE_O_TRACECLONE
            | Options::PTRACE_O_TRACEEXIT;

        //TODO: seccomp
        ptrace::setoptions(self.pid, default_options).context("Failed to set ptrace options")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::fork_test;
    use crate::{filesystem::FileSystem, utils::tests::get_test_rootfs};
    use nix::unistd::Pid;

    #[test]
    fn create_tracee() {
        let tracee = Tracee::new(Pid::from_raw(42), FileSystem::new());
        assert_eq!(tracee.pid, Pid::from_raw(42));
    }

    #[test]
    /// Tests that the set_ptrace_options runs without panicking.
    /// It requires a traced child process to be applied on,
    /// as using `ptrace(PTRACE_SETOPTIONS)` without preparation results in a
    /// Sys(ESRCH) error.
    fn create_set_ptrace_options() {
        let rootfs_path = get_test_rootfs();

        fork_test(
            rootfs_path,
            // expecting a normal execution
            0,
            // parent
            |_, _| {
                // we stop on the first syscall;
                // the fact that no panic was sparked until now
                // means that the set_trace_options call was OK
                true
            },
            // child
            || {},
        );
    }
}
