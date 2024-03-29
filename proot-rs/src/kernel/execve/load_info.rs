use crate::errors::*;
use crate::filesystem::readers::ExtraReader;
use crate::filesystem::FileSystem;
use crate::filesystem::Translator;
use crate::kernel::execve::binfmt::elf::{ElfHeader, ExecutableClass, ProgramHeader};
use crate::kernel::execve::binfmt::elf::{PF_R, PF_W, PF_X, PT_GNU_STACK, PT_INTERP, PT_LOAD};
use crate::register::Word;
use nix::sys::mman::MapFlags;
use nix::sys::mman::ProtFlags;
use nix::unistd::{sysconf, SysconfVar};
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub struct Mapping {
    pub addr: Word,
    pub length: Word,
    pub clear_length: Word,
    pub prot: ProtFlags,
    pub flags: MapFlags,
    pub fd: Option<Word>,
    pub offset: Word,
}

// TODO: redesign this struct and remove unnecessary `Option`
#[derive(Debug, PartialEq)]
pub struct LoadInfo {
    pub raw_path: Option<PathBuf>,
    pub user_path: Option<PathBuf>,
    pub host_path: Option<PathBuf>,
    pub elf_header: ElfHeader,
    pub mappings: Vec<Mapping>,
    pub interp: Option<Box<LoadInfo>>,
    pub needs_executable_stack: bool,
}

lazy_static! {
    static ref PAGE_SIZE: Word = match sysconf(SysconfVar::PAGE_SIZE) {
        Ok(Some(value)) => value as Word,
        _ => 0x1000,
    };
    static ref PAGE_MASK: Word = !(*PAGE_SIZE - 1);
}

//TODO: move these in arch.rs and do cfg for each env
#[cfg(target_arch = "x86")]
const EXEC_PIC_ADDRESS: Word = 0x0f000000;
#[cfg(target_arch = "x86")]
const INTERP_PIC_ADDRESS: Word = 0xaf000000;

#[cfg(target_arch = "x86_64")]
const EXEC_PIC_ADDRESS: Word = 0x500000000000;
#[cfg(target_arch = "x86_64")]
const INTERP_PIC_ADDRESS: Word = 0x6f0000000000;
#[cfg(target_arch = "x86_64")]
const EXEC_PIC_ADDRESS_32: Word = 0x0f000000;
#[cfg(target_arch = "x86_64")]
const INTERP_PIC_ADDRESS_32: Word = 0xaf000000;

#[cfg(target_arch = "arm")]
const EXEC_PIC_ADDRESS: Word = 0x0f000000;
#[cfg(target_arch = "arm")]
const INTERP_PIC_ADDRESS: Word = 0x1f000000;

#[cfg(target_arch = "aarch64")]
const EXEC_PIC_ADDRESS: Word = 0x3000000000;
#[cfg(target_arch = "aarch64")]
const INTERP_PIC_ADDRESS: Word = 0x3f00000000;

impl LoadInfo {
    fn new(elf_header: ElfHeader) -> Self {
        Self {
            raw_path: None,
            user_path: None,
            host_path: None,
            elf_header: elf_header,
            mappings: Vec::new(),
            interp: None,
            needs_executable_stack: false,
        }
    }

    /// Extracts information about an executable:
    /// - the ELF header info
    /// - the program header segments, which contain:
    ///     - mappings
    ///     - interp???
    pub fn from(fs: &FileSystem, host_path: &Path) -> Result<LoadInfo> {
        let mut file = File::open(host_path)?;
        let (elf_header, mut file) = ElfHeader::extract_from(&mut file)?;

        // Sanity checks.
        apply!(elf_header, |header| header.is_exec_or_dyn())?;
        apply!(elf_header, |header| header.is_known_phentsize())?;

        let executable_class = elf_header.get_class();
        let program_headers_offset = get!(elf_header, e_phoff, u64)?;
        let program_headers_count = get!(elf_header, e_phnum)?;

        // We skip the initial part, directly to the program headers.
        file.seek(SeekFrom::Start(program_headers_offset))?;

        let mut load_info = LoadInfo::new(elf_header);

        // We will read all the program headers, and extract info from them.
        for _ in 0..program_headers_count {
            let program_header = match executable_class {
                ExecutableClass::Class32 => ProgramHeader::ProgramHeader32(file.read_struct()?),
                ExecutableClass::Class64 => ProgramHeader::ProgramHeader64(file.read_struct()?),
            };

            let segment_type = get!(program_header, p_type)?;

            match segment_type {
                // Loadable segment. The bytes from the file are mapped to the beginning of the
                // memory segment
                PT_LOAD => load_info.add_mapping(&program_header)?,
                // Specifies the location and size of a null-terminated path name to invoke as an
                // interpreter.
                PT_INTERP => load_info.add_interp(fs, &program_header, &mut file)?,
                // Check if the stack of this executable file is executable (NX disabled)
                PT_GNU_STACK => {
                    let flags = get!(program_header, p_flags)?;
                    let prot = process_prot_flags(flags);
                    load_info.needs_executable_stack = prot.contains(ProtFlags::PROT_EXEC);
                }
                _ => (),
            };
        }

        Ok(load_info)
    }

    /// Processes a program header segment into a Mapping,
    /// which is then added to the mappings list.
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
            // TODO: This can be optimized.
            // The calculation of the `length` may be wrong. The field `length` should not be
            // calculated using the paged-aligned `end_address`. It will cause more content in the
            // target file to be mapped into the memory area. Due to `clear_length`, the
            // over-mapped area is later cleared by `clear_length`. However, according to the man
            // page of mmap(2), the remaining bytes of a file mapping will be zeroed automatically.
            // So the best way is to correct the calculation of `length` and remove `clear_length`
            // field.
            length: end_address - start_address,
            flags: MapFlags::MAP_PRIVATE | MapFlags::MAP_FIXED,
            prot: prot,
            clear_length: 0,
        };

        // According to the description in man page elf(5), `p_filesz` may not be larger
        // than the `p_memsz`.

        // "If the segment's memory size p_memsz is larger than the
        // file size p_filesz, the "extra" bytes are defined to hold
        // the value 0 and to follow the segment's initialized area."
        // -- man 5 elf.
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
                    flags: MapFlags::MAP_PRIVATE | MapFlags::MAP_ANONYMOUS | MapFlags::MAP_FIXED,
                    prot: prot,
                };

                self.mappings.push(new_mapping);
            }
        } else {
            self.mappings.push(mapping);
        }

        Ok(())
    }

    fn add_interp(
        &mut self,
        fs: &FileSystem,
        program_header: &ProgramHeader,
        file: &mut File,
    ) -> Result<()> {
        // Only one PT_INTERP segment is allowed.
        if self.interp.is_some() {
            return Err(Error::errno_with_msg(
                EINVAL,
                "when translating execve, double interp",
            ));
        }

        let user_path_size = get!(program_header, p_filesz, usize)?;
        let user_path_offset = get!(program_header, p_offset, u64)?;
        // the -1 is to avoid the null char `\0`
        let user_path = file.pread_path_at(user_path_size - 1, user_path_offset)?;

        //TODO: implement load info for QEMU
        //        /* When a QEMU command was specified:
        //         *
        //         * - if it's a foreign binary we are reading the ELF
        //         * interpreter of QEMU instead.
        //         *
        //         * - if it's a host binary, we are reading its ELF
        //         * interpreter.
        //         *
        //         * In both case, it lies in "/host-rootfs" from a guest
        //         * point-of-view.  */
        //        if (tracee->qemu != NULL && user_path[0] == '/') {
        //            user_path = talloc_asprintf(tracee->ctx, "%s%s", HOST_ROOTFS,
        // user_path);            if (user_path == NULL)
        //                return -ENOMEM;
        //        }

        let host_path = fs.translate_path(&user_path, true)?.1;
        FileSystem::check_host_path_executable(&host_path)?;

        let mut load_info = LoadInfo::from(fs, &host_path)?;

        load_info.host_path = Some(host_path);
        load_info.user_path = Some(user_path);

        self.interp = Some(Box::new(load_info));

        Ok(())
    }

    /// Add @load_base to each adresses of @load_info.
    #[inline]
    fn add_load_base(&mut self, load_base: Word) -> Result<()> {
        for mapping in &mut self.mappings {
            mapping.addr += load_base;
        }

        self.elf_header.apply_mut(
            |mut header32| {
                header32.e_entry += load_base as u32;
                Ok(())
            },
            |mut header64| {
                header64.e_entry += load_base as u64;
                Ok(())
            },
        )
    }

    /// Compute the final load address for each position independent objects of
    /// @tracee.
    pub fn compute_load_addresses(&mut self, is_interp: bool) -> Result<()> {
        let is_pos_indep = apply!(self.elf_header, |header| header.is_position_independent())?;

        if is_pos_indep && self.mappings.get(0).unwrap().addr == 0 {
            #[cfg(target_arch = "x86_64")]
            let load_base_32 = match is_interp {
                false => EXEC_PIC_ADDRESS_32,  // exec
                true => INTERP_PIC_ADDRESS_32, // interp
            };

            let load_base = match is_interp {
                false => EXEC_PIC_ADDRESS,  // exec
                true => INTERP_PIC_ADDRESS, // interp
            };

            #[cfg(target_arch = "x86_64")]
            if self.elf_header.get_class() == ExecutableClass::Class32 {
                self.add_load_base(load_base_32)?;
            } else {
                self.add_load_base(load_base)?;
            }

            #[cfg(not(target_arch = "x86_64"))]
            self.add_load_base(load_base)?;
        }

        if !is_interp {
            if let Some(ref mut interp_load_info) = self.interp {
                interp_load_info.compute_load_addresses(true)?;
            }
        }
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
    let read_flag = process_flag(flags, PF_R, ProtFlags::PROT_READ, ProtFlags::PROT_NONE);
    let write_flag = process_flag(flags, PF_W, ProtFlags::PROT_WRITE, ProtFlags::PROT_NONE);
    let execute_flag = process_flag(flags, PF_X, ProtFlags::PROT_EXEC, ProtFlags::PROT_NONE);

    read_flag | write_flag | execute_flag
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Error;
    use crate::filesystem::FileSystem;
    use crate::register::Word;
    use crate::utils::tests::get_test_rootfs_path;
    use std::path::PathBuf;

    #[test]
    fn test_load_info_from_invalid_path() {
        let rootfs_path = get_test_rootfs_path();

        let fs = FileSystem::with_root(rootfs_path).unwrap();
        let result = LoadInfo::from(&fs, &PathBuf::from("/../../.."));

        assert!(result.is_err());
        assert_eq!(Error::errno(EISDIR), result.unwrap_err());
    }

    #[test]
    fn test_load_info_from_path_not_executable() {
        let rootfs_path = get_test_rootfs_path();

        let fs = FileSystem::with_root(&rootfs_path).unwrap();
        let result = LoadInfo::from(&fs, &rootfs_path.join("etc/passwd"));

        assert_eq!(Err(Error::errno(ENOEXEC)), result);
    }

    #[test]
    fn test_load_info_from_path_has_mappings() {
        let rootfs_path = get_test_rootfs_path();

        let fs = FileSystem::with_root(&rootfs_path).unwrap();
        let result = LoadInfo::from(&fs, &rootfs_path.join("bin/sleep"));

        assert!(result.is_ok());

        let load_info = result.unwrap();

        assert!(!load_info.mappings.is_empty());
    }

    #[test]
    fn test_load_info_from_path_has_interp() {
        let rootfs_path = get_test_rootfs_path();

        let fs = FileSystem::with_root(&rootfs_path).unwrap();
        let result = LoadInfo::from(&fs, &rootfs_path.join("bin/sleep"));

        assert!(result.is_ok());

        let load_info = result.unwrap();

        assert!(load_info.interp.is_some());

        let interp = load_info.interp.unwrap();

        assert!(interp.host_path.is_some());
        assert!(interp.user_path.is_some());
    }

    #[test]
    #[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
    fn test_load_info_compute_load_addresses() {
        let rootfs_path = get_test_rootfs_path();

        let fs = FileSystem::with_root(&rootfs_path).unwrap();
        let result = LoadInfo::from(&fs, &rootfs_path.join("bin/sleep"));
        let load_info = result.unwrap();
        let mut interp = load_info.interp.unwrap();

        let before_e_entry = get!(interp.elf_header, e_entry, Word).unwrap();

        interp.compute_load_addresses(true).unwrap();

        let after_e_entry = get!(interp.elf_header, e_entry, Word).unwrap();

        assert!(after_e_entry > before_e_entry);
    }
}
