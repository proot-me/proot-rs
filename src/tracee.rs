extern crate nix;
use nix::sys::ioctl::libc::pid_t;
#[allow(non_camel_case_types)]
type word_t = i32;
use nix::sys::ioctl::libc::size_t;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Binding {
    host: PathBuf,
    guest: PathBuf,
    need_substitution: bool,
    must_exist: bool
}

impl Binding {
    pub fn new(host: &str, guest: &str, need_substitution: bool, must_exist: bool) -> Binding {
        Binding {
            host: PathBuf::from(host),
            guest: PathBuf::from(guest),
            need_substitution: need_substitution,
            must_exist: must_exist
        }
    }
}

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

/// Virtual heap, emulated with a regular memory mapping.
#[derive(Debug)]
struct Heap {
    base: Option<word_t>,
    size: Option<size_t>,
    prealloc_size: Option<size_t>
}

impl Heap {
    pub fn new() -> Heap {
        Heap {
            base: None,
            size: None,
            prealloc_size: None
        }
    }
}

#[derive(Debug)]
pub struct Tracee {
    /// Process identifier.
    pid: pid_t,
    /*
    /// Is it running or not?
    running: bool,
    /// Is this tracee ready to be freed?
    terminated: bool,
    /// Parent of this tracee.
    parent: Cell<Option<Tracee>>,
    /// Is it a "clone", i.e has the same parent as its creator.
    clone: bool,
    // as_ptracer,
    // as_ptracee,
    /// Current status:
    ///       0: enter syscall
    ///       1: exit syscall no error
    ///  -errno: exit syscall with error.
    status: i32,
    // reconf,
    // chain,
    // load_info,
    /// Verbose level.
    verbose: i32,
    // seccomp
    /// Ensure the sysexit stage is always hit under seccomp.
    sys_exit_pending: bool,
    */
    /// Information related to a file-system name-space.
    fs: FileSystemNameSpace,
    /// Virtual heap, emulated with a regular memory mapping.
    heap: Heap,
    /*
    /// Path to the executable, à la /proc/self/exe.
    exe: PathBuf,
    new_exe: PathBuf,
    // qemu: OsStr,
    // glue: OsStr,
    // extensions,
    // host_ldso_paths,
    // guest_ldso_paths,
    // tool_name,
    */
}

impl Tracee {

    pub fn new(pid: pid_t, fs: FileSystemNameSpace) -> Tracee {
        Tracee {
            pid: pid,
            heap: Heap::new(),
            fs: fs,
            /*
            running: false,
            terminated: false,
            parent: Cell::new(None),
            clone: false,
            status: 0,
            verbose: 0,
            sys_exit_pending: false,
            exe: Cell::new(None),
            new_exe: Cell::new(None),
            */
        }
    }

    /*
     * UTILITIES
     */

    //pub fn get_pid(& self) -> pid_t { self.pid }
    //pub fn set_pid(&mut self, pid: pid_t) { self.pid = pid; }

    /*
    pub fn is_in_sys_enter(&self) -> bool {
        self.status == 0
    }

    pub fn is_in_sys_exit(&self) -> bool {
        !self.is_in_sys_enter()
    }
    */

    //pub fn is_in_sys_exit2(&self) -> bool {
    //    self.is_in_sys_exit() && get_sysnum((tracee), ORIGINAL) == sysnum)
    //}
}