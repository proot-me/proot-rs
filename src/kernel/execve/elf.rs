
const EI_NIDENT: usize = 16;

/// Use T = u32 for 32bits, and T = u64 for 64bits
#[repr(C)]
pub struct ElfHeader<T> {
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
    e_shstrndx: u16
}

/// Use T = u32 for 32bits, and T = u64 for 64bits
#[repr(C)]
pub struct ProgramHeader<T> {
    p_type: u32,
    p_flags: u32,
    p_offset: T,
    p_vaddr: T,
    p_paddr: T,
    p_filesz: T,
    p_memsz: T,
    p_align: T
}

pub enum SegmentType {
    PtLoad = 1,
    PtDynamic = 2,
    PtInterp = 3
}

/// Use TSigned = i32 and TUnsigned = u32 for 32bits,
/// and TSigned = u64 and TUnsigned = u64 for 64bits
pub struct DynamicEntry<TSigned, TUnsigned> {
    d_tag: TSigned,
    d_val: TUnsigned
}

pub enum DynamicType {
    DtStrtab = 5,
    DtRpath = 15,
    DtRunpath = 29
}


