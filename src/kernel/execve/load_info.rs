use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Seek, SeekFrom};
use errors::Result;
use filesystem::readers::StructReader;
use kernel::execve::elf::{ElfHeader, ProgramHeader, ExecutableClass};

#[derive(Clone, Debug, PartialEq)]
pub struct LoadInfo {
    pub raw_path: Option<PathBuf>,
    pub user_path: Option<PathBuf>,
    pub host_path: Option<PathBuf>,
    pub elf_header: Option<ElfHeader>,
}

impl LoadInfo {
    pub fn new() -> Self {
        Self {
            raw_path: None,
            user_path: None,
            host_path: None,
            elf_header: None,
        }
    }

    pub fn extract_info(&mut self, host_path: &Path) -> Result<()> {
        let mut file = File::open(host_path)?;
        let (elf_header, mut file) = ElfHeader::extract_from(&mut file)?;

        self.elf_header = Some(elf_header);

        // Sanity checks.
        apply!(self.elf_header, |header| header.is_exec_or_dyn())?;
        apply!(self.elf_header, |header| header.is_known_phentsize())?;

        let program_headers_offset = get!(self.elf_header, e_phoff, u64)?;
        let program_headers_count = get!(self.elf_header, e_phnum)?;

        // We skip the initial part, directly to the program headers.
        file.seek(SeekFrom::Start(program_headers_offset))?;

        println!("{:?}", self.elf_header);

        // We will read all the program headers, and extract info from them.
        for _ in 0..program_headers_count {
            let program_header = match self.elf_header.unwrap().get_class() {
                ExecutableClass::Class32 => ProgramHeader::ProgramHeader32(file.read_struct()?),
                ExecutableClass::Class64 => ProgramHeader::ProgramHeader64(file.read_struct()?)
            };

            println!("{:?}", program_header);
        }

        Ok(())
    }
}
