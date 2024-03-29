use crate::errors::*;
use crate::filesystem::readers::ExtraReader;
use crate::filesystem::FileSystem;
use crate::kernel::execve::load_info::LoadInfo;
use crate::kernel::execve::params::ExecveParameters;
use std::any::TypeId;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem;

use super::LoadResult;

const EI_NIDENT: usize = 16;
const ET_REL: u16 = 1;
const ET_EXEC: u16 = 2;
const ET_DYN: u16 = 3;
const ET_CORE: u16 = 4;
pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;
pub const PT_GNU_STACK: u32 = 0x6474_e551;
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

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ProgramHeader32 {
    pub p_type: u32,
    pub p_offset: u32,
    pub p_vaddr: u32,
    pub p_paddr: u32,
    pub p_filesz: u32,
    pub p_memsz: u32,
    pub p_flags: u32,
    pub p_align: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct ProgramHeader64 {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

#[derive(Debug, PartialEq)]
pub enum ProgramHeader {
    ProgramHeader32(ProgramHeader32),
    ProgramHeader64(ProgramHeader64),
}

impl ProgramHeader {
    #[inline]
    pub fn apply<
        V,
        F32: FnOnce(&ProgramHeader32) -> Result<V>,
        F64: FnOnce(&ProgramHeader64) -> Result<V>,
    >(
        &self,
        func32: F32,
        func64: F64,
    ) -> Result<V> {
        match self {
            ProgramHeader::ProgramHeader32(ref program_header) => func32(program_header),
            ProgramHeader::ProgramHeader64(ref program_header) => func64(program_header),
        }
    }
}

/// Use T = u32 for 32bits, and T = u64 for 64bits.
#[repr(C)]
#[derive(Debug, PartialEq)]
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
    pub e_phnum: u16,     // program headers count
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

impl<T: 'static> ParameterizedElfHeader<T> {
    #[inline]
    pub fn is_exec_or_dyn(&self) -> Result<()> {
        match self.e_type {
            self::ET_EXEC | self::ET_DYN => Ok(()),
            o => Err(Error::errno_with_msg(
                EINVAL,
                format!(
                    "ELF type mismatch, ET_EXEC | ET_DYN expected, but got {}",
                    o
                ),
            )),
        }
    }

    #[inline]
    pub fn is_known_phentsize(&self) -> Result<()> {
        let program_header_size = if TypeId::of::<T>() == TypeId::of::<u64>() {
            mem::size_of::<ProgramHeader64>() as u16
        } else {
            mem::size_of::<ProgramHeader32>() as u16
        };

        match self.e_phentsize == program_header_size {
            true => Ok(()),
            false => {
                // note(tracee, WARNING, INTERNAL, "%d: unsupported size of program header.",
                // fd);
                Err(Error::errno_with_msg(
                    EOPNOTSUPP,
                    format!(
                        "Program header size mismatch, {} expected, got {}",
                        program_header_size, self.e_phentsize
                    ),
                ))
            }
        }
    }

    #[inline]
    pub fn is_position_independent(&self) -> Result<bool> {
        Ok(self.e_type == self::ET_DYN)
    }
}

#[derive(Debug, PartialEq)]
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
    #[inline]
    pub fn extract_from(file: &mut File) -> Result<(Self, &mut File)> {
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
    #[inline]
    fn extract_class(file: &mut File) -> Result<(ExecutableClass, &mut File)> {
        let mut buffer = [0; 5];

        file.read_exact(&mut buffer)?;

        match buffer {
            // 0x7f, E, L, F, executable_class
            [0x7f, 69, 76, 70, exe_class] => match exe_class as i32 {
                1 => Ok((ExecutableClass::Class32, file)),
                2 => Ok((ExecutableClass::Class64, file)),
                _ => Err(Error::errno_with_msg(
                    ENOEXEC,
                    format!(
                        "Extracting ELF from unknown executable class: {:X?}",
                        exe_class
                    ),
                )),
            },
            _ => Err(Error::errno_with_msg(
                ENOEXEC,
                format!("Extracting ELF from non executable file: {:X?}", buffer),
            )),
        }
    }

    #[inline]
    pub fn get_class(&self) -> ExecutableClass {
        match self {
            ElfHeader::ElfHeader32(_) => ExecutableClass::Class32,
            ElfHeader::ElfHeader64(_) => ExecutableClass::Class64,
        }
    }

    #[inline]
    pub fn apply<
        V,
        F32: FnOnce(&ParameterizedElfHeader<u32>) -> Result<V>,
        F64: FnOnce(&ParameterizedElfHeader<u64>) -> Result<V>,
    >(
        &self,
        func32: F32,
        func64: F64,
    ) -> Result<V> {
        match self {
            ElfHeader::ElfHeader32(ref elf_header) => func32(elf_header),
            ElfHeader::ElfHeader64(ref elf_header) => func64(elf_header),
        }
    }
    #[inline]
    pub fn apply_mut<
        V,
        F32: FnOnce(&mut ParameterizedElfHeader<u32>) -> Result<V>,
        F64: FnOnce(&mut ParameterizedElfHeader<u64>) -> Result<V>,
    >(
        &mut self,
        func32: F32,
        func64: F64,
    ) -> Result<V> {
        match self {
            ElfHeader::ElfHeader32(ref mut elf_header) => func32(elf_header),
            ElfHeader::ElfHeader64(ref mut elf_header) => func64(elf_header),
        }
    }
}

/// The loader function for regular elf executable file.
pub(super) fn load_elf(fs: &FileSystem, parameters: &mut ExecveParameters) -> Result<LoadResult> {
    // parse LoadInfo from the binary file to be executed
    let mut load_info = LoadInfo::from(fs, &parameters.host_path)
        .with_context(|| format!("Failed to parse elf file: {:?}", parameters.host_path))?;

    load_info.raw_path = Some(parameters.raw_guest_path.clone());
    load_info.user_path = Some(parameters.canonical_guest_path.clone());
    load_info.host_path = Some(parameters.host_path.clone());

    // An ELF interpreter is supposed to be standalone and should not depend on
    // another ELF interpreter.
    if let Some(ref interp) = load_info.interp {
        if interp.interp.is_some() {
            return Err(Error::errno_with_msg(
                EINVAL,
                "When translating enter execve, interpreter of ELF is supposed to be statically linked.",
            ));
        }
    }

    load_info.compute_load_addresses(false)?;
    Ok(LoadResult::Finished(load_info))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Error;
    use std::path::PathBuf;

    #[test]
    fn test_get_elf_header_class_not_executable() {
        let mut file = File::open(PathBuf::from("/etc/hostname")).unwrap();
        assert_eq!(
            ElfHeader::extract_class(&mut file).unwrap_err(),
            Error::errno(ENOEXEC)
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

        assert!(
            get!(elf_header, e_ident).unwrap()[4] == 1 // 32 bit elf file
                || get!(elf_header, e_ident).unwrap()[4] == 2 // 64 bit elf file
        );
        assert!(apply!(elf_header, |header| header.is_exec_or_dyn()).is_ok());
        assert!(apply!(elf_header, |header| header.is_known_phentsize()).is_ok());
    }
}
