use std::path::{Path, PathBuf};
use errors::Result;

use kernel::execve::elf;

#[derive(Clone, Debug, PartialEq)]
pub struct LoadInfo {
    pub raw_path: Option<PathBuf>,
    pub user_path: Option<PathBuf>,
    pub host_path: Option<PathBuf>,
    pub elf_header: Option<elf::ElfHeader>
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

    pub fn extract_info<'a>(&mut self, host_path: &Path) -> Result<()> {
        self.elf_header = Some(elf::extract_elf_head(host_path)?);

        // Sanity check.
        self.elf_header.unwrap().apply(
            |header32| header32.is_exec_or_dyn(),
            |header64| header64.is_exec_or_dyn()
        )?;

        self.iterate_program_header()
    }

    fn iterate_program_header(&mut self) -> Result<()> {
        let elf_phnum = get!(self.elf_header, e_phnum)?;
        let elf_phentsize = get!(self.elf_header, e_phentsize)?;
        let elf_phoff = get!(self.elf_header, e_phoff, u64)?;

        // Sanity check.
        self.elf_header.unwrap().apply(
            |header32| header32.is_known_phentsize(),
            |header64| header64.is_known_phentsize()
        )?;


        Ok(())
    }
}

