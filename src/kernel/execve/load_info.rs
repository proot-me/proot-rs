use std::path::{Path, PathBuf};
use errors::Result;
use kernel::execve::elf;

#[derive(Clone, Debug, PartialEq)]
pub struct LoadInfo {
    pub raw_path: Option<PathBuf>,
    pub user_path: Option<PathBuf>,
    pub host_path: Option<PathBuf>,
    pub elf_header: Option<elf::ElfHeader>,
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
}

pub fn extract<'a>(host_path: &Path, load_info: &'a mut LoadInfo) -> Result<&'a mut LoadInfo> {
    let elf_header = elf::extract_elf_head(host_path)?;

    // Sanity check.
    elf_header.apply(
        |header| header.is_exec_or_dyn(),
        |header| header.is_exec_or_dyn(),
    )?;

    Ok(load_info)
}
