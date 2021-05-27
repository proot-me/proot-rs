use crate::filesystem::binding::Binding;
use crate::filesystem::validation::{binding_validator, path_validator};
use crate::filesystem::FileSystem;
use clap::{App, Arg};
use std::path::PathBuf;

pub const DEFAULT_ROOTFS: &'static str = "/";
pub const DEFAULT_CWD: &'static str = ".";

pub fn parse_config() -> (FileSystem, Vec<String>) {
    let mut fs: FileSystem = FileSystem::new();

    let matches = App::new("proot-rsc")
        .arg(Arg::with_name("rootfs")
            .short("r")
            .long("rootfs")
            .help("Use *path* as the new guest root file-system.")
            .takes_value(true)
            .default_value(DEFAULT_ROOTFS)
            .validator(path_validator))
        .arg(Arg::with_name("bind")
            .short("b")
            .long("bind")
            .help("Make the content of *host_path* accessible in the guest rootfs. Format: host_path:guest_path")
            .multiple(true)
            .takes_value(true)
            .validator(binding_validator))
        .arg(Arg::with_name("cwd")
            .short("w")
            .long("cwd")
            .help("Set the initial working directory to *path*.")
            .takes_value(true)
            .default_value(DEFAULT_CWD))
        .arg(Arg::with_name("command")
            .multiple(true))
        .get_matches();

    debug!("proot-rs startup with args:\n{:#?}", matches);

    // option -r
    let rootfs: &str = matches.value_of("rootfs").unwrap();
    // -r *path* is equivalent to -b *path*:/
    fs.set_root(rootfs);

    // option(s) -b
    if let Some(bindings) = matches.values_of("bind") {
        let raw_bindings_str: Vec<&str> = bindings.collect::<Vec<&str>>();

        for raw_binding_str in &raw_bindings_str {
            let parts: Vec<&str> = raw_binding_str.split_terminator(':').collect();
            fs.add_binding(Binding::new(parts[0], parts[1], true));
        }
    }

    // option -w
    let cwd: &str = matches.value_of("cwd").unwrap();
    fs.set_cwd(PathBuf::from(cwd));

    // command
    let command: Vec<String> = match matches.values_of("command") {
        Some(values) => values.map(|s| s.into()).collect(),
        None => ["/bin/sh".into()].into(),
    };

    (fs, command)
}
