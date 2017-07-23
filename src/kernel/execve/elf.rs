use std::fs::File;
use std::io::{SeekFrom, Seek, Read};
use std::mem;
use errors::{Error, Result};
use filesystem::readers::ExtraReader;

const EI_NIDENT: usize = 16;
const ET_REL: u16 = 1;
const ET_EXEC: u16 = 2;
const ET_DYN: u16 = 3;
const ET_CORE: u16 = 4;
pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;
pub const PF_X: u32 = 1;
pub const PF_W: u32 = 2;
pub const PF_R: u32 = 4;


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
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: T,
    pub p_vaddr: T,
    pub p_paddr: T,
    pub p_filesz: T,
    pub p_memsz: T,
    pub p_align: T,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProgramHeader {
    ProgramHeader32(ParameterizedProgramHeader<u32>),
    ProgramHeader64(ParameterizedProgramHeader<u64>),
}

impl ProgramHeader {
    pub fn apply<
        V,
        F32: FnOnce(ParameterizedProgramHeader<u32>) -> Result<V>,
        F64: FnOnce(ParameterizedProgramHeader<u64>) -> Result<V>,
    >(
        &self,
        func32: F32,
        func64: F64,
    ) -> Result<V> {
        match *self {
            ProgramHeader::ProgramHeader32(program_header) => func32(program_header),
            ProgramHeader::ProgramHeader64(program_header) => func64(program_header),
        }
    }
}

/// Use T = u32 for 32bits, and T = u64 for 64bits.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterizedElfHeader<T> {
    pub e_ident: [u8; EI_NIDENT], // identifier; it should start with ['\x7f', 'E', 'L', 'F'].
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: T,
    pub e_phoff: T, // program header offset
    pub e_shoff: T,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16, // program header entire size
    pub e_phnum: u16, // program headers count
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

impl<T> ParameterizedElfHeader<T> {
    pub fn is_exec_or_dyn(&self) -> Result<()> {
        match self.e_type {
            self::ET_EXEC | self::ET_DYN => Ok(()),
            _ => Err(Error::invalid_argument("when checking elf header type, not supported type")),
        }
    }

    pub fn is_known_phentsize(&self) -> Result<()> {
        let program_header_size = mem::size_of::<ParameterizedProgramHeader<T>>() as u16;

        match self.e_phentsize == program_header_size {
            true => Ok(()),
            false => {
                // note(tracee, WARNING, INTERNAL, "%d: unsupported size of program header.", fd);
                Err(Error::not_supported("when checking program header size, mismatch with struct size"))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ElfHeader {
    ElfHeader32(ParameterizedElfHeader<u32>),
    ElfHeader64(ParameterizedElfHeader<u64>),
}

impl ElfHeader {
    /// Extracts the ElfHeader structure from the file, if possible.
    ///
    /// Returns an error if something happened (`io::Error`),
    /// `None` if it's not an ELF-executable,
    /// and an `ElfHeader` if it was successful.
    pub fn extract_from<'a>(file: &'a mut File) -> Result<(Self, &'a mut File)> {
        let (executable_class, file) = ElfHeader::extract_class(file)?;

        // we reset the file's iterator
        file.seek(SeekFrom::Start(0))?;

        let elf_header = match executable_class {
            ExecutableClass::Class32 => ElfHeader::ElfHeader32(file.read_struct()?),
            ExecutableClass::Class64 => ElfHeader::ElfHeader64(file.read_struct()?),
        };

        Ok((elf_header, file))
    }

    /// Reads the five first characters of a file,
    /// to determine whether or not it's an ELF executable,
    /// and whether the executable is 32 or 64 bits.
    fn extract_class<'a>(file: &'a mut File) -> Result<(ExecutableClass, &'a mut File)> {
        let mut buffer = [0; 5];

        file.read_exact(&mut buffer)?;

        match buffer {
            [0x7f, 69, 76, 70, exe_class] => {
                match exe_class as i32 {
                    1 => Ok((ExecutableClass::Class32, file)),
                    2 => Ok((ExecutableClass::Class64, file)),
                    _ => Err(Error::cant_exec("when extracting elf from unknown executable class")),
                }
            }
            _ => Err(Error::cant_exec("when extracting elf header from non executable file")),
        }
    }

    pub fn get_class(&self) -> ExecutableClass {
        match *self {
            ElfHeader::ElfHeader32(_) => ExecutableClass::Class32,
            ElfHeader::ElfHeader64(_) => ExecutableClass::Class64,
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
    fn test_get_elf_header_class_not_executable() {
        let mut file = File::open(PathBuf::from("/etc/init/acpid.conf")).unwrap();
        assert_eq!(
            ElfHeader::extract_class(&mut file).unwrap_err(),
            Error::cant_exec("when extracting elf header from non executable file")
        );
    }

    #[test]
    fn test_get_elf_header_class() {
        let mut file = File::open(PathBuf::from("/bin/sleep")).unwrap();
        assert!(ElfHeader::extract_class(&mut file).is_ok());
    }

    #[test]
    fn test_extract_elf_header() {
        let mut file = File::open(PathBuf::from("/bin/sleep")).unwrap();
        let (elf_header, _) = ElfHeader::extract_from(&mut file).unwrap();

        assert_eq!(get!(elf_header, e_ident).unwrap()[4], 2);
        assert!(apply!(elf_header, |header| header.is_exec_or_dyn()).is_ok());
        assert!(apply!(elf_header, |header| header.is_known_phentsize()).is_ok());
    }
}
