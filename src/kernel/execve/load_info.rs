use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Seek, SeekFrom};
use nix::unistd::{sysconf, SysconfVar};
use nix::sys::mman::{MapFlags, MAP_PRIVATE, MAP_FIXED, MAP_ANONYMOUS};
use nix::sys::mman::{ProtFlags, PROT_NONE, PROT_READ, PROT_WRITE, PROT_EXEC};
use errors::Result;
use register::Word;
use filesystem::readers::StructReader;
use kernel::execve::elf::{PT_LOAD, PT_INTERP, PF_R, PF_W, PF_X};
use kernel::execve::elf::{ElfHeader, ProgramHeader, ExecutableClass};

#[derive(Clone, Debug, PartialEq)]
pub struct Mapping {
    addr: Word,
    length: Word,
    clear_length: Word,
    prot: ProtFlags,
    flags: MapFlags,
    fd: Option<Word>,
    offset: Word,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadInfo {
    pub raw_path: Option<PathBuf>,
    pub user_path: Option<PathBuf>,
    pub host_path: Option<PathBuf>,
    pub elf_header: ElfHeader,
    pub mappings: Vec<Mapping>
}

lazy_static! {
    static ref PAGE_SIZE: Word = match sysconf(SysconfVar::PAGE_SIZE) {
            Ok(Some(value)) => value as Word,
            _ => 0x1000
        };
    static ref PAGE_MASK: Word = !(*PAGE_SIZE - 1);
}


impl LoadInfo {
    fn new(elf_header: ElfHeader) -> Self {
        Self {
            raw_path: None,
            user_path: None,
            host_path: None,
            elf_header: elf_header,
            mappings: Vec::new()
        }
    }

    pub fn from(host_path: &Path) -> Result<LoadInfo> {
        let mut file = File::open(host_path)?;
        let (elf_header, mut file) = ElfHeader::extract_from(&mut file)?;

        // Sanity checks.
        apply!(elf_header, |header| header.is_exec_or_dyn())?;
        apply!(elf_header, |header| header.is_known_phentsize())?;

        let program_headers_offset = get!(elf_header, e_phoff, u64)?;
        let program_headers_count = get!(elf_header, e_phnum)?;

        // We skip the initial part, directly to the program headers.
        file.seek(SeekFrom::Start(program_headers_offset))?;

        let mut load_info = LoadInfo::new(elf_header);

        // We will read all the program headers, and extract info from them.
        for _ in 0..program_headers_count {
            let program_header = match elf_header.get_class() {
                ExecutableClass::Class32 => ProgramHeader::ProgramHeader32(file.read_struct()?),
                ExecutableClass::Class64 => ProgramHeader::ProgramHeader64(file.read_struct()?),
            };

            let segment_type = get!(program_header, p_type)?;

            match segment_type {
                PT_LOAD => load_info.add_mapping(&program_header)?,
                PT_INTERP => load_info.add_interp(&program_header)?,
                _ => ()
            };
        }

        Ok(load_info)
    }

    fn add_mapping(&mut self, program_header: &ProgramHeader) -> Result<()> {
        let vaddr = get!(program_header, p_vaddr, Word)?;
        let memsz = get!(program_header, p_memsz, Word)?;
        let filesz = get!(program_header, p_filesz, Word)?;
        let offset = get!(program_header, p_offset, Word)?;
        let flags = get!(program_header, p_flags)?;

        let start_address = vaddr & *PAGE_MASK;
        let end_address = (vaddr + filesz + *PAGE_SIZE) & *PAGE_MASK;
        let prot = process_prot_flags(flags);

        let mut mapping = Mapping {
            fd: None, // unknown yet
            offset: offset & *PAGE_MASK,
            addr: start_address,
            length: end_address - start_address,
            flags: MAP_PRIVATE | MAP_FIXED,
            prot: prot,
            clear_length: 0,
        };

        // "If the segment's memory size p_memsz is larger than the
        // file size p_filesz, the "extra" bytes are defined to hold
        // the value 0 and to follow the segment's initialized area."
        // -- man 7 elf.
        if memsz > filesz {
            // How many extra bytes in the current page?
            mapping.clear_length = end_address - vaddr - filesz;

            self.mappings.push(mapping);

            let start_address = end_address;
            let end_address = (vaddr + memsz + *PAGE_SIZE) & *PAGE_MASK;

            // Create new pages for the remaining extra bytes.
            if end_address > start_address {
                let new_mapping = Mapping {
                    fd: None,
                    offset: 0,
                    addr: start_address,
                    length: end_address - start_address,
                    clear_length: 0,
                    flags: MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED,
                    prot: prot
                };

                self.mappings.push(new_mapping);
            }
        } else {
            self.mappings.push(mapping);
        }

        Ok(())
    }


    fn add_interp(&mut self, program_header: &ProgramHeader) -> Result<()> {
        println!("interp: {:?}", program_header);

        Ok(())
    }
}

#[inline]
fn process_flag<T>(flags: u32, compare_flag: u32, success_flag: T, default_flag: T) -> T {
    if flags & compare_flag > 0 {
        success_flag
    } else {
        default_flag
    }
}

#[inline]
fn process_prot_flags(flags: u32) -> ProtFlags {
    let read_flag = process_flag(flags, PF_R, PROT_READ, PROT_NONE);
    let write_flag = process_flag(flags, PF_W, PROT_WRITE, PROT_NONE);
    let execute_flag = process_flag(flags, PF_X, PROT_EXEC, PROT_NONE);

    read_flag | write_flag | execute_flag
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use errors::Error;

    #[test]
    fn test_load_info_from_invalid_path() {
        let result = LoadInfo::from(&PathBuf::from("/../../.."));

        assert!(result.is_err());
        assert_eq!(Error::is_a_directory(), result.unwrap_err());
    }

    #[test]
    fn test_load_info_from_path_not_executable() {
        let result = LoadInfo::from(&PathBuf::from("/etc/init/acpid.conf"));

        assert!(result.is_err());
        assert_eq!(Error::cant_exec(), result.unwrap_err());
    }

    #[test]
    fn test_load_info_from_path_has_mappings() {
        let result = LoadInfo::from(&PathBuf::from("/bin/sleep"));

        assert!(result.is_ok());

        let load_info = result.unwrap();

        assert!(load_info.mappings.len() > 0);
    }
}