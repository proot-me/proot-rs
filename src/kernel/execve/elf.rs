use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::mem;
use errors::{Error, Result};
use filesystem::utils::StructReader;

const EI_NIDENT: usize = 16;
const ET_REL: u16 = 1;
const ET_EXEC: u16 = 2;
const ET_DYN: u16 = 3;
const ET_CORE: u16 = 4;

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
pub enum ExecutableClass {
    Class32 = 1,
    Class64 = 2,
}

/// Use T = u32 for 32bits, and T = u64 for 64bits.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterizedProgramHeader<T> {
    p_type: u32,
    p_flags: u32,
    p_offset: T,
    p_vaddr: T,
    p_paddr: T,
    p_filesz: T,
    p_memsz: T,
    p_align: T,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProgramHeader {
    ProgramHeader32(ParameterizedProgramHeader<u32>),
    ProgramHeader64(ParameterizedProgramHeader<u64>)
}

/// Use T = u32 for 32bits, and T = u64 for 64bits.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterizedElfHeader<T> {
    e_ident: [u8; EI_NIDENT], // identifier; it should start with ['\x7f', 'E', 'L', 'F'].
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: T,
    pub e_phoff: T, // program header offset
    e_shoff: T,
    e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16, // program header entire size
    pub e_phnum: u16, // program headers count
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

impl<T> ParameterizedElfHeader<T> {
    pub fn is_exec_or_dyn(&self) -> Result<()> {
        match self.e_type {
            self::ET_EXEC | self::ET_DYN => Ok(()),
            _ => Err(Error::invalid_argument()),
        }
    }

    pub fn is_known_phentsize(&self) -> Result<()> {
        let program_header_size = mem::size_of::<ParameterizedProgramHeader<T>>() as u16;

        match self.e_phentsize == program_header_size {
            true => Ok(()),
            false => {
                // note(tracee, WARNING, INTERNAL, "%d: unsupported size of program header.", fd);
                Err(Error::not_supported())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ElfHeader {
    ElfHeader32(ParameterizedElfHeader<u32>),
    ElfHeader64(ParameterizedElfHeader<u64>)
}

impl ElfHeader {
    /// Extracts the ElfHeader structure from the file, if possible.
    ///
    /// Returns an error if something happened (`io::Error`),
    /// `None` if it's not an ELF-executable,
    /// and an `ElfHeader` if it was successful.
    pub fn extract_from(path: &Path) -> Result<Self> {
        let executable_class = Self::extract_class(path)?;
        let mut file = File::open(path)?;

        let elf_header = match executable_class {
            ExecutableClass::Class32 => ElfHeader::ElfHeader32(file.read_struct()?),
            ExecutableClass::Class64 => ElfHeader::ElfHeader64(file.read_struct()?),
        };

        Ok(elf_header)
    }

    /// Reads the five first characters of a file,
    /// to determine whether or not it's an ELF executable,
    /// and whether the executable is 32 or 64 bits.
    fn extract_class(path: &Path) -> Result<ExecutableClass> {
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

    pub fn get_class(&self) -> ExecutableClass {
        match *self {
            ElfHeader::ElfHeader32(_) => ExecutableClass::Class32,
            ElfHeader::ElfHeader64(_) => ExecutableClass::Class64
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use errors::Error;

    #[test]
    fn test_get_elf_header_class_no_file_error() {
        let result = ElfHeader::extract_class(&PathBuf::from("/../../test"));

        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(Error::no_such_file_or_dir(), err);
        }
    }

    #[test]
    fn test_get_elf_header_class_not_executable() {
        assert!(ElfHeader::extract_class(&PathBuf::from("/etc/init/acpid.conf")).is_err());
    }

    #[test]
    fn test_get_elf_header_class() {
        assert!(ElfHeader::extract_class(&PathBuf::from("/bin/sleep")).is_ok());
    }

    #[test]
    fn test_extract_elf_header() {
        let result = ElfHeader::extract_from(&PathBuf::from("/bin/sleep"));

        assert!(result.is_ok());

        let elf_header = result.unwrap();

        assert_eq!(get!(Some(elf_header), e_ident).unwrap()[4], 2);
        assert!(apply!(Some(elf_header), |header| header.is_exec_or_dyn()).is_ok());
        assert!(apply!(Some(elf_header), |header| header.is_known_phentsize()).is_ok());
    }
}
