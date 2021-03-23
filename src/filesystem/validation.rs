use std::path::Path;

//TODO: replace all this by FileSystemNamespace's trait Validator

/// Check wheter the path is a valid path (file that exists, or path that ends in /)
pub fn is_valid_path(path: &str, error_message: String) -> Result<(), String> {
    if !Path::new(path).exists() {
        Err(error_message)
    } else {
        Ok(())
    }
}

/// Check whether the path exists and is a folder
pub fn path_validator(path: String) -> Result<(), String> {
    is_valid_path(path.as_str(), path.to_string() + " is not a valid path.")
    //TODO: check for folder path
}

/// Check whether a path is of the type ```host_path:guest_path``` and that the host.
pub fn binding_validator(binding_paths: String) -> Result<(), String> {
    let parts: Vec<&str> = binding_paths.split_terminator(':').collect();

    if parts.len() != 2 {
        Err("should be: path_host:path_guest".to_string())
    } else {
        let host_path: &str = parts[0];

        is_valid_path(host_path, host_path.to_string() + " is not a valid path.")
    }

    //TODO: add a check to avoid equivalent paths bindings?
    //TODO: add a check for guest path? (rootfs + guest path must exists?)
    //TODO: add a check to check both paths are of the same type (file:file or folder:folder)
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
        let incorrect_paths = [
            "impossible path",
            "../../../../impossible path",
            "/\\/",
            "\'`",
        ];

        for path in &incorrect_paths {
            assert_eq!(
                path_validator(path.to_string()),
                Err(path.to_string() + " is not a valid path.")
            );
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
            assert_eq!(
                binding_validator(path.to_string()),
                Err("should be: path_host:path_guest".to_string())
            );
        }
        assert_eq!(
            binding_validator("impossible path:.".to_string()),
            Err("impossible path is not a valid path.".to_string())
        );
    }
}
