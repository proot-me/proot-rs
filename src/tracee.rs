extern crate nix;
use nix::sys::ioctl::libc::pid_t;
#[allow(non_camel_case_types)]
type word_t = i32;
use nix::sys::ioctl::libc::size_t;
use std::path::PathBuf;
use std::cell::Cell;

#[derive(Debug)]
struct Binding {
    host: PathBuf,
    guest: PathBuf,
    need_substitution: bool,
    must_exist: bool
}

#[derive(Debug)]
struct FileSystemNameSpaceBindings {
    /// List of bindings as specified by the user but not canonicalized yet.
    pending: Vec<Binding>,
    /// List of bindings canonicalized and sorted in the "guest" order.
    guest: Vec<Binding>,
    /// List of bindings canonicalized and sorted in the "host" order.
    host: Vec<Binding>
}

impl FileSystemNameSpaceBindings {
    pub fn new() -> FileSystemNameSpaceBindings {
        FileSystemNameSpaceBindings {
            pending: vec![],
            guest: vec![],
            host: vec![]
        }
    }
}

/// Information related to a file-system name-space.
#[derive(Debug)]
struct FileSystemNameSpace {
    bindings: FileSystemNameSpaceBindings,
    /// Current working directory, à la /proc/self/pwd.
    cwd: Option<PathBuf>
}

impl FileSystemNameSpace {
    pub fn new() -> FileSystemNameSpace {
        FileSystemNameSpace {
            bindings: FileSystemNameSpaceBindings::new(),
            cwd: None
        }
    }
}

/// Virtual heap, emulated with a regular memory mapping.
#[derive(Debug)]
struct Heap {
    base: Cell<Option<word_t>>,
    size: Cell<Option<size_t>>,
    prealloc_size: Cell<Option<size_t>>
}

impl Heap {
    pub fn new() -> Heap {
        Heap {
            base: Cell::new(None),
            size: Cell::new(None),
            prealloc_size: Cell::new(None)
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
    fs: Box<FileSystemNameSpace>,
    /// Virtual heap, emulated with a regular memory mapping.
    heap: Box<Heap>,
    /*
    /// Path to the executable, à la /proc/self/exe.
    exe: Cell<Option<OsStr>>,
    new_exe: Cell<Option<OsStr>>,
    // qemu: OsStr,
    // glue: OsStr,
    // extensions,
    // host_ldso_paths,
    // guest_ldso_paths,
    // tool_name,
    */
}

impl Tracee {

    pub fn new(pid: pid_t) -> Tracee {
        Tracee {
            pid: pid,
            heap: Box::new(Heap::new()),
            fs: Box::new(FileSystemNameSpace::new()),
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