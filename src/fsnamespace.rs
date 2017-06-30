use std::path::PathBuf;
use bindings::Binding;


/// Information related to a file-system name-space.
#[derive(Debug)]
pub struct FileSystemNamespace {
    bindings: Vec<Binding>,
    /// Current working directory, à la /proc/self/pwd.
    cwd: PathBuf
}

impl FileSystemNamespace {
    pub fn new() -> FileSystemNamespace {
        FileSystemNamespace {
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