use clap::{App, Arg};
use std::string::String;
use std::path::{Path};
use tracee::{Binding, FileSystemNameSpace};

const DEFAULT_ROOTFS: &'static str = "/";
const DEFAULT_CWD: &'static str = ".";

/// Check wheter the path is a valid path (file that exists, or path that ends in /)
fn is_valid_path(path: &str, error_message: String) -> Result<(), String> {
    if !Path::new(path).exists() {
        Err(error_message)
    } else {
        Ok(())
    }
}

/// Check whether the path exists and is a folder
fn path_validator(path: String) -> Result<(), String> {
    is_valid_path(path.as_str(), path.to_string() + " is not a valid path.")
    //TODO: check for folder path
}

/// Check whether a path is of the type ```host_path:guest_path``` and that the host.
fn binding_validator(binding_paths: String) -> Result<(), String> {
    let parts: Vec<&str> = binding_paths.split_terminator(":").collect();

    if parts.len() != 2 {
        Err(("should be: path_host:path_guest".to_string()))
    } else {
        let host_path: &str = parts[0];

        is_valid_path(host_path, host_path.to_string() + " is not a valid path.")
    }

    //TODO: add a check to avoid equivalent paths bindings?
    //TODO: add a check for guest path? (rootfs + guest path must exists?)
    //TODO: add a check to check both paths are of the same type (file:file or folder:folder)
}

pub fn get_config(fs: &mut FileSystemNameSpace) {
    let matches = App::new("proot_rust")
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
        .get_matches();

    // option -r
    let rootfs: &str = matches.value_of("rootfs").unwrap();
    // -r *path* is equivalent to -b *path*:/
    fs.add_binding(Binding::new(rootfs, "/", true, true));

    // option(s) -b
    match matches.values_of("bind") {
        Some(b_bindings) => {
            let raw_bindings_str: Vec<&str> = b_bindings.collect::<Vec<&str>>();

            for raw_binding_str in &raw_bindings_str {
                let parts: Vec<&str> = raw_binding_str.split_terminator(":").collect();
                fs.add_binding(Binding::new(parts[0], parts[1], true, true));
            }
        },
        None    => ()
    };

    // option -w
    let cwd: &str = matches.value_of("cwd").unwrap();
    fs.set_cwd(cwd);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_validator_correct_paths() {
        let correct_paths = [".", "./", "..", "../", "./.."];

        for path in &correct_paths {
            assert_eq!(path_validator(path.to_string()), Ok(()));
        }
    }

    #[test]
    fn test_path_validator_incorrect_paths() {
        let incorrect_paths = ["impossible path", "../../../../impossible path", "/\\/", "\'`"];

        for path in &incorrect_paths {
            assert_eq!(path_validator(path.to_string()), Err((path.to_string() + " is not a valid path.")));
        }
    }

    #[test]
    fn test_binding_validator_correct_bindings() {
        let correct_bindings = [".:.", "..:..", ".:../../", ".:ignored"];

        for path in &correct_bindings {
            assert_eq!(binding_validator(path.to_string()), Ok(()));
        }
    }

    #[test]
    fn test_binding_validator_incorrect_bindings() {
        let incorrect_paths = [".", "..", "..:..:..", ".:.:."];

        for path in &incorrect_paths {
            assert_eq!(binding_validator(path.to_string()), Err(("should be: path_host:path_guest".to_string())));
        }
        assert_eq!(binding_validator("impossible path:.".to_string()), Err(("impossible path is not a valid path.".to_string())));
    }
}