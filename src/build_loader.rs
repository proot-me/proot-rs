extern crate gcc;

#[cfg(target_arch = "x86")]
mod arch {
    pub const LOADER_ADDRESS: u64 = 0xa0000000;
    pub const LOADER_ARCH_CFLAGS: &[&'static str] = &["-mregparm=3"];
}

#[cfg(target_arch = "x86_64")]
mod arch {
    /// The virtual address(p_vaddr) of `.text` section of our custom loader.
    pub const LOADER_ADDRESS: u64 = 0x600000000000;
    /// Additional flags to be passed when compiling the loader.
    pub const LOADER_ARCH_CFLAGS: &[&'static str] = &[];
}

#[cfg(target_arch = "arm")]
mod arch {
    pub const LOADER_ADDRESS: u64 = 0x10000000;
    pub const LOADER_ARCH_CFLAGS: &[&'static str] = &[];
}

#[cfg(target_arch = "aarch64")]
mod arch {
    pub const LOADER_ADDRESS: u64 = 0x2000000000;
    pub const LOADER_ARCH_CFLAGS: &[&'static str] = &[];
}

fn main() {
    let mut config = gcc::Config::new();
    config
        .flag("-static")
        .flag("-nostdlib")
        .flag("-ffreestanding");
    arch::LOADER_ARCH_CFLAGS.iter().for_each(|flag| {
        config.flag(flag);
    });
    config.flag(&format!("-Wl,-Ttext=0x{:x}", arch::LOADER_ADDRESS));
    config
        .file("src/kernel/execve/loader/loader.c")
        .out_dir("src/kernel/execve/loader")
        .compile_binary("binary_loader_exe");
}
