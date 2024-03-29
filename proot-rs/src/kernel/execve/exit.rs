use std::os::unix::prelude::OsStrExt;

use libc::c_void;
use loader_shim::script::LoadStatement;
use loader_shim::script::LoadStatementMmap;
use loader_shim::script::LoadStatementOpen;
use loader_shim::script::LoadStatementStackExec;
use loader_shim::script::LoadStatementStart;
use nix::sys::mman::MapFlags;
use nix::unistd::SysconfVar;

use crate::errors::Result;
use crate::process::tracee::Tracee;
use crate::register::PtraceWriter;
use crate::register::Word;
use crate::register::{Current, StackPointer, SysArg, SysArgIndex, SysResult};

pub fn translate(tracee: &mut Tracee) -> Result<()> {
    let syscall_result = tracee.regs.get(Current, SysResult) as isize;

    //TODO: implement ptrace execve exit translation

    if syscall_result < 0 {
        return Ok(());
    }

    if tracee.new_exe.is_some() {
        // Execve happened; commit the new "/proc/self/exe".
        tracee.exe = tracee.new_exe.take();
    }

    //TODO: implement heap
    // New processes have no heap.
    //bzero(tracee->heap, sizeof(Heap));

    let res = transfert_load_script(tracee);
    tracee.load_info = None;
    res
}

pub fn transfert_load_script(tracee: &mut Tracee) -> Result<()> {
    // the original stack pointer value
    let stack_pointer = tracee.regs.get(Current, StackPointer) as usize;

    let load_info = tracee.load_info.as_ref().unwrap();
    // collect strings
    let string1_bytes = load_info.user_path.as_ref().unwrap().as_os_str().as_bytes();
    let string1_size = string1_bytes.len() + 1;
    let string2_bytes = load_info
        .interp
        .as_ref()
        .map(|interp| interp.user_path.as_ref().unwrap().as_os_str().as_bytes());
    let string2_size = string2_bytes.map_or(0, |s| s.len() + 1);
    let string3_bytes = if load_info.user_path == load_info.raw_path {
        None
    } else {
        Some(load_info.raw_path.as_ref().unwrap().as_os_str().as_bytes())
    };
    let string3_size = string3_bytes.map_or(0, |s| s.len() + 1);

    // we need to make sure fields are aligned, so we calculate padding size.
    let padding_size =
        (stack_pointer - string1_size - string2_size - string3_size) % tracee.sizeof_word();
    let strings_size = string1_size + string2_size + string3_size + padding_size;
    let string1_address = stack_pointer - strings_size;
    let string2_address = stack_pointer - strings_size + string1_size;
    let string3_address = if string3_size == 0 {
        string1_address
    } else {
        stack_pointer - strings_size + string1_size + string2_size
    };

    // create a buffer to store the data we need to write to tracee's stack
    let mut buffer: Vec<u8> = vec![];
    // write load statement: open
    let stmt = LoadStatement::Open(LoadStatementOpen {
        string_address: string1_address as Word,
    });
    debug!("LoadStatement: {:x?}", stmt);
    buffer.extend_from_slice(stmt.as_bytes());
    // write load statement: mmap
    for mapping in &load_info.mappings {
        let stmt = if mapping.flags.contains(MapFlags::MAP_ANONYMOUS) {
            LoadStatement::MmapAnonymous(LoadStatementMmap {
                addr: mapping.addr,
                length: mapping.length,
                prot: mapping.prot.bits() as libc::c_ulong,
                offset: mapping.offset,
                clear_length: mapping.clear_length,
            })
        } else {
            LoadStatement::MmapFile(LoadStatementMmap {
                addr: mapping.addr,
                length: mapping.length,
                prot: mapping.prot.bits() as libc::c_ulong,
                offset: mapping.offset,
                clear_length: mapping.clear_length,
            })
        };
        debug!("LoadStatement: {:x?}", stmt);
        buffer.extend_from_slice(stmt.as_bytes());
    }
    // if interpreter exist, we also need to load for interpreter (PT_INTERP)
    if let Some(interp) = load_info.interp.as_ref() {
        // write load statement: open next
        let stmt = LoadStatement::OpenNext(LoadStatementOpen {
            string_address: string2_address as libc::c_ulong,
        });
        debug!("LoadStatement: {:x?}", stmt);
        buffer.extend_from_slice(stmt.as_bytes());
        // write load statement: mmap
        for mapping in &interp.mappings {
            let stmt = if mapping.flags.contains(MapFlags::MAP_ANONYMOUS) {
                LoadStatement::MmapAnonymous(LoadStatementMmap {
                    addr: mapping.addr,
                    length: mapping.length,
                    prot: mapping.prot.bits() as libc::c_ulong,
                    offset: mapping.offset,
                    clear_length: mapping.clear_length,
                })
            } else {
                LoadStatement::MmapFile(LoadStatementMmap {
                    addr: mapping.addr,
                    length: mapping.length,
                    prot: mapping.prot.bits() as libc::c_ulong,
                    offset: mapping.offset,
                    clear_length: mapping.clear_length,
                })
            };
            debug!("LoadStatement: {:x?}", stmt);
            buffer.extend_from_slice(stmt.as_bytes());
        }
    }

    // If the stack of the executable file or it's interpreter is marked as
    // executable (NX disabled), then we need to reset the stack to executable
    if load_info.needs_executable_stack
        || (load_info.interp.is_some() && load_info.interp.as_ref().unwrap().needs_executable_stack)
    {
        // if any error occurs or the page size cannot be got, we use 0x1000 as default
        // value
        let page_size = nix::unistd::sysconf(SysconfVar::PAGE_SIZE)
            .unwrap_or(None)
            .unwrap_or(0x1000);
        let page_mask = !(page_size - 1) as usize;

        let stmt = LoadStatement::MakeStackExec(LoadStatementStackExec {
            start: (stack_pointer & page_mask) as libc::c_ulong,
        });
        debug!("LoadStatement: {:x?}", stmt);
        buffer.extend_from_slice(stmt.as_bytes());
    }

    // determine the entry_point of this executable
    let entry_point = if let Some(interp) = load_info.interp.as_ref() {
        get!(interp.elf_header, e_entry, libc::c_ulong)?
    } else {
        get!(load_info.elf_header, e_entry, libc::c_ulong)?
    };

    // Load script statement: start.
    // TODO: Start of the program slightly differs when ptraced. see proot https://github.com/proot-me/proot/blob/fb9503240eeaa3114b29b8742feb2bda6edccde8/src/execve/exit.c#L298
    let stmt = LoadStatement::Start(LoadStatementStart {
        stack_pointer: stack_pointer as libc::c_ulong,
        entry_point: entry_point,
        at_phdr: get!(load_info.elf_header, e_phoff, libc::c_ulong)? + load_info.mappings[0].addr,
        at_phent: get!(load_info.elf_header, e_phentsize, libc::c_ulong)?,
        at_phnum: get!(load_info.elf_header, e_phnum, libc::c_ulong)?,
        at_entry: get!(load_info.elf_header, e_entry, libc::c_ulong)?,
        at_execfn: string3_address as libc::c_ulong,
    });
    debug!("LoadStatement: {:x?}", stmt);
    buffer.extend_from_slice(stmt.as_bytes());

    // TODO: consider 32on64 mode: https://github.com/proot-me/proot/blob/fb9503240eeaa3114b29b8742feb2bda6edccde8/src/execve/exit.c#L319

    // Concatenate the load script and the strings.
    buffer.extend_from_slice(string1_bytes);
    buffer.push(b'\0');
    if string2_size != 0 {
        buffer.extend_from_slice(string2_bytes.unwrap());
        buffer.push(b'\0');
    }
    if string3_size != 0 {
        buffer.extend_from_slice(string3_bytes.unwrap());
        buffer.push(b'\0');
    }

    // write load script to the stack space of tracee, and set value of stack
    // pointer and first argument
    let new_stack_pointer = stack_pointer - padding_size - buffer.len();
    tracee
        .regs
        .write_data(new_stack_pointer as *mut c_void, &buffer, false)?;
    tracee.regs.set(
        StackPointer,
        new_stack_pointer as libc::c_ulong,
        "update stack pointer address in execve::exit()",
    );
    tracee.regs.set(
        SysArg(SysArgIndex::SysArg1),
        new_stack_pointer as libc::c_ulong,
        "update stack pointer address in execve::exit()",
    );

    // we need to make sure current register values will be used as-is at the end.
    tracee.regs.set_restore_original_regs(false);
    Ok(())
}
