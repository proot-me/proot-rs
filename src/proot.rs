use tracee::{Tracee, FileSystemNameSpace };
use nix::sys::ioctl::libc::pid_t;
use std::collections::HashMap;

#[derive(Debug)]
pub struct PRoot {
    tracees: HashMap<pid_t, Tracee>,
}

// Main memory of the program
impl PRoot {
    pub fn new() -> PRoot {
        PRoot {
            tracees: HashMap::new()
        }
    }

    pub fn create_tracee(&mut self, pid: pid_t, fs: FileSystemNameSpace) -> Option<&Tracee>  {
        self.tracees.insert(pid, Tracee::new(pid, fs));

        return self.get_tracee(pid);
    }

    pub fn get_tracee(&self, pid: pid_t) -> Option<&Tracee> {
        return self.tracees.get(&pid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_tracee() {
        let mut proot = PRoot::new();
        let fs = FileSystemNameSpace::new();

        // tracee 0 shouldn't exist
        assert!(proot.get_tracee(0).is_none());

        { proot.create_tracee(0, fs); }

        // tracee 0 should exist
        assert!(proot.get_tracee(0).is_some());
    }
}