use errors::{Error, Result};
use std::io;
use std::mem;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::slice;

const EI_NIDENT: usize = 16;

/// Use T = u32 for 32bits, and T = u64 for 64bits.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterizedElfHeader<T> {
    e_ident: [u8; EI_NIDENT],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: T,
    e_phoff: T,
    e_shoff: T,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

const ET_REL: u16 = 1;
const ET_EXEC: u16 = 2;
const ET_DYN: u16 = 3;
const ET_CORE: u16 = 4;

impl<T> ParameterizedElfHeader<T> {
    pub fn is_exec_or_dyn(&self) -> Result<()> {
        match self.e_type {
            self::ET_EXEC | self::ET_DYN => Ok(()),
            _ => Err(Error::invalid_argument()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ElfHeader {
    ElfHeader32(ParameterizedElfHeader<u32>),
    ElfHeader64(ParameterizedElfHeader<u64>),
}

impl ElfHeader {
    pub fn apply<
        V,
        F32: FnOnce(ParameterizedElfHeader<u32>) -> Result<V>,
        F64: FnOnce(ParameterizedElfHeader<u64>) -> Result<V>,
    >(
        &self,
        func32: F32,
        func64: F64,
    ) -> Result<V> {
        match *self {
            ElfHeader::ElfHeader32(elf_header) => func32(elf_header),
            ElfHeader::ElfHeader64(elf_header) => func64(elf_header),
        }
    }
}

/// Use T = u32 for 32bits, and T = u64 for 64bits.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgramHeader<T> {
    p_type: u32,
    p_flags: u32,
    p_offset: T,
    p_vaddr: T,
    p_paddr: T,
    p_filesz: T,
    p_memsz: T,
    p_align: T,
}

pub enum SegmentType {
    PtLoad = 1,
    PtDynamic = 2,
    PtInterp = 3,
}

/// Use TSigned = i32 and TUnsigned = u32 for 32bits,
/// and TSigned = u64 and TUnsigned = u64 for 64bits
pub struct DynamicEntry<TSigned, TUnsigned> {
    d_tag: TSigned,
    d_val: TUnsigned,
}

pub enum DynamicType {
    DtStrtab = 5,
    DtRpath = 15,
    DtRunpath = 29,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ExecutableClass {
    Class32 = 1,
    Class64 = 2,
}

/// Extracts the ElfHeader structure from the file, if possible.
///
/// Returns an error if something happened (`io::Error`),
/// `None` if it's not an ELF-executable,
/// and an `ElfHeader` if it was successful.
pub fn extract_elf_head(path: &Path) -> Result<ElfHeader> {
    let executable_class = get_elf_header_class(path)?;

    let elf_header = match executable_class {
        ExecutableClass::Class32 => ElfHeader::ElfHeader32(read_struct(path)?),
        ExecutableClass::Class64 => ElfHeader::ElfHeader64(read_struct(path)?),
    };

    Ok(elf_header)
}

/// Reads the five first characters of a file,
/// to determine whether or not it's an ELF executable,
/// and whether the executable is 32 or 64 bits.
fn get_elf_header_class(path: &Path) -> Result<ExecutableClass> {
    let file = File::open(path)?;
    let mut chars = file.chars();

    match (
        chars.next().unwrap()?,
        chars.next().unwrap()?,
        chars.next().unwrap()?,
        chars.next().unwrap()?,
        chars.next().unwrap()?,
    ) {
        ('\x7f', 'E', 'L', 'F', exe_class) => {
            match exe_class as i32 {
                1 => Ok(ExecutableClass::Class32),
                2 => Ok(ExecutableClass::Class64),
                _ => Err(Error::cant_exec()),
            }
        }
        _ => Err(Error::cant_exec()),
    }
}

/// Reads the context of a file, and extracts + transmutes its content into a structure.
fn read_struct<T>(path: &Path) -> io::Result<T> {
    let mut file = File::open(path)?;
    let num_bytes = mem::size_of::<T>();
    unsafe {
        let mut s = mem::uninitialized();
        let mut buffer = slice::from_raw_parts_mut(&mut s as *mut T as *mut u8, num_bytes);
        match file.read_exact(buffer) {
            Ok(()) => Ok(s),
            Err(e) => {
                ::std::mem::forget(s);
                Err(e)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use errors::Error;

    #[test]
    fn test_get_elf_header_class_no_file_error() {
        let result = get_elf_header_class(&PathBuf::from("/../../test"));

        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(Error::no_such_file_or_dir(), err);
        }
    }

    #[test]
    fn test_get_elf_header_class_not_executable() {
        assert!(get_elf_header_class(&PathBuf::from("/etc/init/acpid.conf")).is_err());
    }

    #[test]
    fn test_get_elf_header_class_64_bits() {
        assert_eq!(
            Ok(ExecutableClass::Class64),
            get_elf_header_class(&PathBuf::from("/bin/sleep"))
        );
    }

    #[test]
    fn test_extract_elf_header_64_bits() {
        let elf_header = extract_elf_head(&PathBuf::from("/bin/sleep")).unwrap();
    }
}
