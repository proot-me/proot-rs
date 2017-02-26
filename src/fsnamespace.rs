use std::path::PathBuf;
use bindings::Binding;

/// Information related to a file-system name-space.
#[derive(Debug)]
pub struct FileSystemNameSpace {
    bindings: Vec<Binding>,
    /// Current working directory, à la /proc/self/pwd.
    cwd: PathBuf
}

impl FileSystemNameSpace {
    pub fn new() -> FileSystemNameSpace {
        FileSystemNameSpace {
            bindings: vec![],
            cwd: PathBuf::from(".")
        }
    }

    pub fn add_binding(&mut self, binding: Binding) {
        self.bindings.push(binding);
    }

    pub fn set_cwd(&mut self, cwd: &str) {
        self.cwd = PathBuf::from(cwd);
    }
}