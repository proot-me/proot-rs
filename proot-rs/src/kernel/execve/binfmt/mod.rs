//! This module contains the definition of the supported executable file type
//! and the load and parse functions for each type of the executable files.
//!
//! We define the corresponding load function for each type of executable and
//! expose a unified load function for other modules. These functions are
//! defined in a way similar to `/fs/binfmt_*.c` in the Linux kernel.

pub mod elf;
pub mod shebang;

use super::load_info::LoadInfo;
use crate::errors::*;
use crate::{filesystem::FileSystem, kernel::execve::params::ExecveParameters};

/// Loading results of executable files, which is returned by each type of
/// loading function.
enum LoadResult {
    /// Loading successfully, with the final `LoadInfo` returned.
    Finished(LoadInfo),
    /// Loading successfully, and interpreter was found or `argv` was updated,
    /// in this case we need to restart loading. With this approach, we can
    /// implement recursive parsing of the interpreter. This means that the
    /// interpreter of a script can still be a script.
    RestartWithNewParameters,
}

/// To avoid infinite loops when parsing the interpreter, we need to set a limit
/// on the amount of rewriting we can do to the interpreter.
///
/// In the Linux kernel, this value is 4. https://elixir.bootlin.com/linux/v5.14-rc3/source/fs/exec.c#L1745
const INTERPRETER_REWRITE_LIMIT: usize = 4;

/// This function designed to solve the problem of loading different types
/// executable files. To load an executable, the external module should call
/// this function instead of the specific load function in this module.
///
/// If an executable is loaded successfully, a `LoadInfo` will be returned for
/// further execution. Note that `parameters` may be modified by loader
/// functions.
pub(super) fn load(fs: &FileSystem, parameters: &mut ExecveParameters) -> Result<LoadInfo> {
    const LOADERS: [fn(&FileSystem, &mut ExecveParameters) -> Result<LoadResult>; 2] =
        [shebang::load_script, elf::load_elf];

    // Limit the number of loads to avoid infinite loops of the interpreter
    for _ in 0..(INTERPRETER_REWRITE_LIMIT + 1) {
        let mut restart = false;
        let mut last_error = None;
        // Iterate through each load function
        for loader_fn in LOADERS.iter() {
            restart = false;
            last_error = None;

            // Update canonical_guest_path and host_path
            parameters.update_path(fs)?;
            let metadata = parameters
                .host_path
                .metadata()
                .errno(ENOENT)
                .with_context(|| format!("file not exist: {:?}", parameters.host_path))?;
            if !metadata.is_file() {
                return Err(Error::errno_with_msg(
                    EACCES,
                    "The file to be executed is not a regular file",
                ));
            }
            // Check if this file is executable
            FileSystem::check_host_path_executable(&parameters.host_path)?;

            match loader_fn(fs, parameters) {
                // New interpreter detected, need to load again
                Ok(LoadResult::RestartWithNewParameters) => {
                    restart = true;
                    break;
                }
                // Load success
                Ok(LoadResult::Finished(load_info)) => return Ok(load_info),
                // Load failed, but we can also try another loader function
                Err(error) => {
                    // If the load error code is `ENOEXEC`, it means that the current load function
                    // cannot be used to load such file format. In this case we have to give the
                    // other load function a chance.
                    if error.get_errno() != ENOEXEC {
                        // The current load function can be used to load a file of this format, but
                        // there is an error when loading. In this case we should not try another
                        // loader functions.
                        return Err(error);
                    } else {
                        // Record this error, and give a chance to other load functions.
                        last_error = Some(error)
                    }
                }
            }
        }
        if restart {
            continue;
        }
        // If all loader functions failed, raise the last error.
        if let Some(error) = last_error {
            return Err(error);
        }
    }

    return Err(Error::errno_with_msg(
        ELOOP,
        "failed to load executable file, max interpreter rewrite limit exceeded",
    ));
}
