use clap::{App, Arg};
use std::string::String;
use std::path::Path;

const DEFAULT_ROOTFS: &'static str = "/";

fn is_valid_path(path: &str, error_message: String) -> Result<(), String> {
    if !Path::new(path).exists() {
        Err(error_message)
    } else {
        Ok(())
    }
}

/// Checks whether the path for the rootfs exists.
fn rootfs_validator(path: String) -> Result<(), String> {
    is_valid_path(path.as_str(), path.to_string() + " is not a valid path.")
}

/// Check whether a path is of the type ```host_path:guest_path```,
/// and that the host
fn binding_validator(binding_paths: String) -> Result<(), String> {
    let parts: Vec<&str> = binding_paths.split_terminator(":").collect();

    if parts.len() != 2 {
        Err(("should be: path_host:path_guest".to_string()))
    } else {
        let host_path: &str = parts[0];

        is_valid_path(host_path, host_path.to_string() + " is not a valid path.")
    }
}

pub fn get_config() {
    let matches = App::new("proot_rust")
        .arg(Arg::with_name("rootfs")
            .short("r")
            .long("rootfs")
            .help("Use *path* as the new guest root file-system.")
            .takes_value(true)
            .default_value(DEFAULT_ROOTFS)
            .validator(rootfs_validator))
        .arg(Arg::with_name("bind")
            .short("b")
            .long("bind")
            .help("Make the content of *host_path* accessible in the guest rootfs. Format: host_path:guest_path")
            .multiple(true)
            .takes_value(true)
            .validator(binding_validator))
        .get_matches();

    let rootfs = matches.value_of("rootfs").unwrap();
    println!("Value for rootfs: {}", rootfs);

    match matches.values_of("bind") {
        Some(bindings) => {
            println!("Value for bindings: {:?}", bindings.collect::<Vec<_>>());
        },
        None    => ()
    };
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rootfs_validator_correct_paths() {
        let correct_paths = [".", "./", "..", "../", "./.."];

        for path in &correct_paths {
            assert_eq!(rootfs_validator(path.to_string()), Ok(()));
        }
    }

    #[test]
    fn test_rootfs_validator_incorrect_paths() {
        let incorrect_paths = ["impossible path", "../../../../impossible path", "/\\/", "\'`"];

        for path in &incorrect_paths {
            assert_eq!(rootfs_validator(path.to_string()), Err((path.to_string() + " is not a valid path.")));
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