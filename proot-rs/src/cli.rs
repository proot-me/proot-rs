use clap::{App, Arg};

use crate::errors::*;
use crate::filesystem::validation::{binding_validator, path_validator};
use crate::filesystem::FileSystem;

pub const DEFAULT_ROOTFS: &'static str = "/";
pub const DEFAULT_CWD: &'static str = "/";

pub fn get_args_parser() -> App<'static, 'static> {
    App::new("proot-rs")
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
}

pub fn parse_config() -> Result<(FileSystem, Vec<String>)> {
    let app = get_args_parser();

    let mut fs: FileSystem = FileSystem::new();

    let matches = app.get_matches();

    debug!("proot-rs startup with args:\n{:#?}", matches);

    // option -r
    let rootfs: &str = matches.value_of("rootfs").unwrap();
    // -r *path* is equivalent to -b *path*:/
    fs.set_root(rootfs)?;

    // option(s) -b
    if let Some(bindings) = matches.values_of("bind") {
        let raw_bindings_str: Vec<&str> = bindings.collect::<Vec<&str>>();

        for raw_binding_str in &raw_bindings_str {
            let parts: Vec<&str> = raw_binding_str.split_terminator(':').collect();
            fs.add_binding(parts[0], parts[1])?;
        }
    }

    // option -w
    let cwd: &str = matches.value_of("cwd").unwrap();
    fs.set_cwd(cwd)?;

    // command
    let command: Vec<String> = match matches.values_of("command") {
        Some(values) => values.map(|s| s.into()).collect(),
        None => ["/bin/sh".into()].into(),
    };

    Ok((fs, command))
}
