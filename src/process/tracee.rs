use std::os::unix::io::RawFd;
use std::path::{Path, PathBuf};

use nix::sys::ptrace::{self, Options};
use nix::sys::signal::Signal;
use nix::unistd::Pid;

use crate::errors::*;
use crate::filesystem::Translator;
use crate::filesystem::{binding::Side, FileSystem};
use crate::kernel::execve::load_info::LoadInfo;
use crate::process::proot::InfoBag;
use crate::register::{Registers, Word};

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
    /// An instance of LoadInfo to record information about current `execve`
    /// system call
    pub load_info: Option<LoadInfo>,
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
            load_info: None,
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

    /// Return the byte size of a Word in tracee
    pub fn sizeof_word(&self) -> usize {
        std::mem::size_of::<Word>()
    }

    /// Get current work directory (cwd)
    /// This function will return a **guest side** **absolute path**
    pub fn get_cwd(&self) -> &Path {
        // TODO: each tracee has their own cwd
        let cwd = self.fs.get_cwd();
        if cwd.is_relative() {
            warn!(
                "cwd of tracee({}) is not absolute, there may be some bugs: {:?}",
                self.pid, cwd
            );
        }
        cwd
    }

    /// Get file path from file descriptor
    pub fn get_path_from_fd(&self, fd: RawFd, side: Side) -> Result<PathBuf> {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            if fd == libc::AT_FDCWD {
                // special fd, which point to cwd
                let guest_path = self.get_cwd();
                Ok(match side {
                    Side::Host => self.fs.translate_path(guest_path, false)?,
                    Side::Guest => guest_path.into(),
                })
            } else {
                let proc_fd = format!("/proc/{}/fd/{}", self.pid, fd);
                let maybe_path = PathBuf::from(nix::fcntl::readlink(proc_fd.as_str())?);
                // The /proc/PID/fd/FD is always a symlink pointing to a absolute file/dir path.
                // If not, the FD must not be a file/dir.
                if !maybe_path.is_absolute() {
                    return Err(Error::errno_with_msg(
                        Errno::EBADF,
                        format!(
                            "The file descriptor is not pointing to a file or directory: {:?}",
                            maybe_path
                        ),
                    ));
                }
                let host_path = maybe_path;
                Ok(match side {
                    Side::Guest => {
                        self.fs.detranslate_path(&host_path, None)?.ok_or_else(|| {
                            Error::errno_with_msg(
                                EBADF,
                                format!(
                                    "path exist but failed to convert to guest side: {:?}",
                                    host_path
                                ),
                            )
                        })?
                    }
                    Side::Host => host_path,
                })
            }
        }
        // TODO: on some Unix which contains no /proc, use fcntl(fd, F_GETPATH,
        // pathbuf) instead.
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
