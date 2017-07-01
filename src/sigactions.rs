// signals
use libc::{pid_t, siginfo_t, c_int, c_void};
use nix::sys::signal::{sigaction, Signal, SigAction, SigSet, SigHandler};
use nix::sys::signal::{SaFlags, SA_SIGINFO, SA_RESTART};
use nix::sys::signal::Signal::*;

/// Configures the actions associated with specific critical signals.
/// All signals are blocked when the signal handler is called.
pub fn prepare_sigactions(
    stop_program: extern "C" fn(c_int, *mut siginfo_t, *mut c_void),
    show_info: extern "C" fn(pid: pid_t)
) {
    let signal_set: SigSet = SigSet::all();
    // SIGINFO is used to know which process has signaled us and
    // RESTART is used to restart waitpid(2) seamlessly.
    let sa_flags: SaFlags = SA_SIGINFO | SA_RESTART;

    for signal in Signal::iterator() {
        let mut signal_handler: SigHandler = SigHandler::SigIgn; // default action is ignoring

        // setting the action when receiving certain signals
        match signal {
            SIGQUIT | SIGILL | SIGABRT | SIGFPE | SIGSEGV => {
                // tracees on abnormal termination signals
                signal_handler = SigHandler::SigAction(stop_program);
            }
            SIGUSR1 | SIGUSR2 => {
                // can be used for inter-process communication
                signal_handler = SigHandler::Handler(show_info);
            }
            SIGCHLD | SIGCONT | SIGTSTP | SIGTTIN | SIGTTOU | SIGSTOP | SIGKILL => {
                // these signals are related to tty and job control,
                // or cannot be used with sigaction (stop and kill),
                // so we keep the default action for them
                continue;
            }
            _ => {} // all other signals (even ^C) are ignored
        }

        let signal_action = SigAction::new(signal_handler, sa_flags, signal_set);
        let sigaction_result = unsafe {sigaction(signal, &signal_action)};

        if let Err(err) = sigaction_result {
            println!("Warning: sigaction failed for signal {:?} : {:?}.", signal, err);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    pub extern "C" fn mock_stop_program(_: c_int, _: *mut siginfo_t, _: *mut c_void) {}
    pub extern "C" fn mock_show_info(_: pid_t) {}

    #[test]
    fn prepare_sigactions_test() {
        // should pass without panicking
        prepare_sigactions(mock_stop_program, mock_show_info);
    }

    //TODO: test show_info
}