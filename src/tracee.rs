use nix::sys::ioctl::libc::pid_t;
use nix::sys::signal::Signal;


#[derive(Debug)]
pub struct Tracee {
    /// Process identifier.
    pid: pid_t
}

impl Tracee {
    pub fn new(pid: pid_t) -> Tracee {
        Tracee {
            pid: pid
        }
    }

    pub fn handle_event(&mut self, stop_signal: Signal) {
        println!("stopped tracee: {:?}", self);
        println!("stop signal: {:?}", stop_signal);
    }
}