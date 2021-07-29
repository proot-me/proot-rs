use crate::errors::*;
use crate::errors::{Error, Result};
use crate::filesystem::FileSystem;
use crate::kernel::execve::params::{Arg, ExecveParameters};
use std::ffi::CString;
use std::os::unix::prelude::OsStrExt;
use std::path::{Path, PathBuf};
use std::{fs::File, io::Read};

use super::LoadResult;

#[derive(Debug, PartialEq, Eq)]
pub struct ExtractResult {
    pub interpreter: PathBuf,
    pub optional_arg: Option<CString>,
}

/// Length of the initial part of the executable to be read.
/// See https://elixir.bootlin.com/linux/v5.14-rc3/source/include/uapi/linux/binfmts.h#L19
const BINPRM_BUF_SIZE: usize = 256;

/// The loader function for script file which contains a shebang.
///
/// The definition of a script file can be found in the document of
/// [`extract()`] function.
///
/// According to man page execve(2). if the original command line to be executed
/// is `filename arg...`, a successful load will result in the command line
/// being replaced with `interpreter [optional-arg] filename arg...`. And the
/// return value of the function will be set to
/// `Ok(LoadResult::RestartWithNewParameters)`.
///
/// Note that this function will modify the value of `parameters`:
///  - Replace argv[0] with the raw guest side path
///    (`parameters.raw_guest_path`).
///  - Append `interpreter` and `optional-arg`(if exists) to the front of argv.
///  - Replace the path of the executable(`parameters.raw_guest_path`) with the
///    path of the `interpreter`.
pub(super) fn load_script(
    _fs: &FileSystem,
    parameters: &mut ExecveParameters,
) -> Result<LoadResult> {
    // Extract shebang from script file
    let extract_result = extract(&parameters.host_path)?;

    // Modify execve parameters
    // First, remove the old of argv[0].
    parameters.argv.remove(0);
    // Insert the path of this script into the front of argv.
    parameters.argv.insert(
        0,
        Arg::CStringInSelf(unsafe {
            CString::from_vec_unchecked(parameters.raw_guest_path.as_os_str().as_bytes().into())
        }),
    );
    // If optional argument exist, also push it to the front.
    if let Some(arg) = extract_result.optional_arg {
        parameters.argv.insert(0, Arg::CStringInSelf(arg));
    }
    // Insert Path of the interpreter into the the front.
    parameters.argv.insert(
        0,
        Arg::CStringInSelf(unsafe {
            CString::from_vec_unchecked(extract_result.interpreter.as_os_str().as_bytes().into())
        }),
    );

    // reset raw_guest_path to the new interpreter.
    parameters.raw_guest_path = extract_result.interpreter;

    return Ok(LoadResult::RestartWithNewParameters);
}

/// This function takes a host-side file path, checks for the presence of '#!'
/// and tries to parse out the interpreter and optional-arg.
///
/// As is defined by man execve(2), a script file staring with a line of the
/// form:
///
/// #!interpreter [optional-arg]
///
/// Where the path of `interpreter` comes after the `#!` (spaces are
/// allowed), and the remainder of the content immediately following is the
/// `optional-arg`. Note that optional-arg is treated as one argument and not as
/// multiple arguments.
fn extract(host_path: &Path) -> Result<ExtractResult> {
    let mut file = File::open(host_path)?;
    let mut buffer = [0u8; BINPRM_BUF_SIZE];
    file.read(&mut buffer)?;

    // refuse to execute this if not start with #!
    match (buffer[0], buffer[1]) {
        (b'#', b'!') => {}
        _ => {
            return Err(Error::errno_with_msg(
                ENOEXEC,
                "file does not start with a shebang",
            ))
        }
    }

    // First, calculate the position of the end of the first line.
    let line_end = buffer
        .iter()
        .position(|&c| c == b'\n')
        .unwrap_or(buffer.len());
    // In the first line, search for the start position of interpreter.
    let interpreter_start = buffer[2..line_end]
        .iter()
        .position(|&c| c != b' ' && c != b'\t')
        .map(|p| 2 + p)
        .ok_or_else(|| Error::errno_with_msg(ENOEXEC, "no interpreter found"))?;
    // search for the end position of interpreter.
    let interpreter_end = buffer[interpreter_start..line_end]
        .iter()
        .position(|&c| c == b' ' || c == b'\t' || c == b'\0')
        .map(|p| interpreter_start + p)
        .unwrap_or(line_end);
    // If the end position of interpreter is the same as the length of buffer, we
    // must assume the interpreter path is truncated, which is not allowed.
    if interpreter_end == buffer.len() {
        return Err(Error::errno_with_msg(
            ENOEXEC,
            "the interpreter path is truncated",
        ));
    }

    // Read interpreter
    if interpreter_start == interpreter_end {
        return Err(Error::errno_with_msg(
            EACCES,
            "path of interpreter is empty",
        ));
    }
    let interpreter = PathBuf::from(
        std::str::from_utf8(&buffer[interpreter_start..interpreter_end])
            .errno(ENOENT)
            .context("path of interpreter is not valid")?,
    );

    // On Linux, the entire string following the interpreter name is passed as a
    // single argument to the interpreter, and this string can include white space.
    // In this case, optional argument is everything from ope_arg_start to the end
    // of line, stripping external white space.

    // Now check if there is an optional argument.
    let opt_arg_start = buffer[interpreter_end..line_end]
        .iter()
        .position(|&c| c != b' ' && c != b'\t')
        .map(|p| interpreter_end + p)
        .unwrap_or(line_end);
    let opt_arg = if opt_arg_start == line_end {
        // There is no optional argument
        None
    } else {
        // Strip white space at the end.
        let line_end = buffer[opt_arg_start..line_end]
            .iter()
            .rposition(|&c| c != b' ' && c != b'\t')
            .map(|p| opt_arg_start + p + 1)
            .unwrap_or(opt_arg_start);

        // Search for the position of next b'\0', as the range of optional argument.
        let opt_arg_end = buffer[opt_arg_start..line_end]
            .iter()
            .position(|&c| c == b'\0')
            .map(|p| opt_arg_start + p)
            .unwrap_or(line_end);

        if opt_arg_end - opt_arg_start > 0 {
            Some(
                CString::new(&buffer[opt_arg_start..opt_arg_end])
                    .errno(ENOEXEC)
                    .context("optional argument is not valid")?,
            )
        } else {
            None
        }
    };

    Ok(ExtractResult {
        interpreter: interpreter,
        optional_arg: opt_arg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::get_test_rootfs_path;

    #[test]
    fn test_extract_shebang_not_script() {
        let rootfs_path = get_test_rootfs_path();

        // it should detect that `/bin/sleep` is not a script
        assert_eq!(
            Err(Error::errno(ENOEXEC)),
            extract(&rootfs_path.join("bin/sleep"))
        );
    }
}
