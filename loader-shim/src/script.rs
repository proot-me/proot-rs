#[cfg(any(target_arch = "x86", target_arch = "arm"))]
pub type Word = u32;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub type Word = u64;

#[cfg_attr(any(target_arch = "x86_64", target_arch = "aarch64"), repr(C, u64))]
#[cfg_attr(any(target_arch = "x86", target_arch = "arm"), repr(C, u32))]
#[allow(dead_code)]
#[derive(Debug)]
pub enum LoadStatement {
    /// Close last opened file and open a new file.
    OpenNext(LoadStatementOpen),
    /// Open a file, and save the file descriptor.
    Open(LoadStatementOpen),
    /// Map a part of the last opened file into memory.
    MmapFile(LoadStatementMmap),
    /// Mapping a segment of anonymous private space into memory instead of mapping from a file.
    MmapAnonymous(LoadStatementMmap),
    /// Set the stack space to be executable
    MakeStackExec(LoadStatementStackExec),
    /// (The purpose of the project is not yet clear)
    StartTraced(LoadStatementStart),
    ///
    Start(LoadStatementStart),
}

#[repr(C)]
#[derive(Debug)]
pub struct LoadStatementOpen {
    pub string_address: Word,
}

#[repr(C)]
#[derive(Debug)]
pub struct LoadStatementMmap {
    /// The starting address for the new mapping.
    pub addr: Word,
    /// The length of the mapping.
    pub length: Word,
    /// The desired memory protection of the mapping.
    pub prot: Word,
    /// The offset in the file which the mapping will start at.
    pub offset: Word,
    /// The byte size of the memory area to be zeroed forward at the end of the
    /// page in this memory mapping.
    pub clear_length: Word,
}

#[repr(C)]
#[derive(Debug)]
pub struct LoadStatementStackExec {
    /// The beginning address (page-aligned) of the stack.
    pub start: Word,
}

#[repr(C)]
#[derive(Debug)]
pub struct LoadStatementStart {
    /// Value of stack pointer
    pub stack_pointer: Word,
    /// The entry address of the executable, or the entry address of the loader if `PT_INTERP` exists.
    pub entry_point: Word,
    pub at_phdr: Word,
    pub at_phent: Word,
    pub at_phnum: Word,
    pub at_entry: Word,
    pub at_execfn: Word,
}

impl LoadStatement {
    pub fn as_bytes(&self) -> &[u8] {
        let mut size = match self {
            LoadStatement::OpenNext(_) | LoadStatement::Open(_) => {
                core::mem::size_of::<LoadStatementOpen>()
            }
            LoadStatement::MmapFile(_) | LoadStatement::MmapAnonymous(_) => {
                core::mem::size_of::<LoadStatementMmap>()
            }
            LoadStatement::MakeStackExec(_) => core::mem::size_of::<LoadStatementStackExec>(),
            LoadStatement::StartTraced(_) | LoadStatement::Start(_) => {
                core::mem::size_of::<LoadStatementStart>()
            }
        };

        size += core::mem::size_of::<Word>();

        let bytes = unsafe {
            core::slice::from_raw_parts((self as *const LoadStatement) as *const u8, size)
        };
        bytes
    }
}
