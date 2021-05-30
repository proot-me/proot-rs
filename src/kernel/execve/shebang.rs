extern crate bstr;

use self::bstr::BStr;
use self::bstr::BString;
use self::bstr::ByteSlice;
use crate::errors::*;
use crate::errors::{Error, Result};
use crate::filesystem::{FileSystem, Translator};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::{fs::File, io::Read};

/// Expand in argv[] the shebang of `user_path`, if any.  This function
/// returns -errno if an error occurred, 1 if a shebang was found and
/// extracted, otherwise 0.  On success, both `host_path` and `user_path`
/// point to the program to execute (respectively from host
/// point-of-view and as-is), and @tracee's argv[] (pointed to by
/// `SYSARG_2`) is correctly updated.
// int expand_shebang(Tracee *tracee, char host_path[PATH_MAX], char
// user_path[PATH_MAX])
pub fn expand(fs: &FileSystem, user_path: &Path) -> Result<PathBuf> {
    //  ArrayOfXPointers *argv = NULL;
    //	bool has_shebang = false;
    //
    //	char argument[BINPRM_BUF_SIZE];
    //	int status;
    //	size_t i;

    // "The interpreter must be a valid pathname for an executable
    //  which is not itself a script [1].  If the filename
    //  argument of execve() specifies an interpreter script, then
    //  interpreter will be invoked with the following arguments:
    //
    //    interpreter [optional-arg] filename arg...
    //
    // where arg...  is the series of words pointed to by the argv
    // argument of execve()." -- man 2 execve
    //
    // [1]: as of this writing (3.10.17) this is true only for the
    //      ELF interpreter; ie. a script can use a script as
    //      interpreter.

    let mut result_host_path: Option<PathBuf> = None;
    let mut loop_iterations = 0;
    let mut has_shebang = false;
    let max_sym_links = 50; //TODO: found this constant in libc

    while loop_iterations < max_sym_links {
        loop_iterations += 1;

        // Translate this path (user -> host), then check it is executable.
        let host_path = translate_and_check_exec(fs, user_path)?;
        let expanded_user_path = extract(&host_path)?;

        if expanded_user_path.is_none() {
            result_host_path = Some(host_path);
            break;
        }
        has_shebang = true;

        // Translate new path (user -> host), then check it is executable.
        let new_host_path = translate_and_check_exec(fs, &expanded_user_path.unwrap())?;

        println!("new host path: {:?}", new_host_path);
    }

    //TODO: implement argument extraction for scripts
    //
    //		/* Fetch argv[] only on demand.  */
    //		if (argv == NULL) {
    //			status = fetch_array_of_xpointers(tracee, &argv, SYSARG_2, 0);
    //			if (status < 0)
    //				return status;
    //		}
    //
    //		/* Assuming the shebang of "script" is "#!/bin/sh -x",
    // 		 * a call to:
    //		 *
    // 		 * execve("./script", { "script.sh", NULL }, ...)
    //		 *
    // 		 * becomes:
    //		 *
    // 		 * execve("/bin/sh", { "/bin/sh", "-x", "./script", NULL }, ...)
    //		 *
    // 		 * See commit 8c8fbe85 about "argv->length == 1".  */
    //		if (argument[0] != '\0') {
    //			status = resize_array_of_xpointers(argv, 0, 2 + (argv->length == 1));
    //			if (status < 0)
    //				return status;
    //
    //			status = write_xpointees(argv, 0, 3, user_path, argument, old_user_path);
    //			if (status < 0)
    //				return status;
    //		}
    //		else {
    //			status = resize_array_of_xpointers(argv, 0, 1 + (argv->length == 1));
    //			if (status < 0)
    //				return status;
    //
    //			status = write_xpointees(argv, 0, 2, user_path, old_user_path);
    //			if (status < 0)
    //				return status;
    //		}
    //	}
    //

    if loop_iterations == max_sym_links {
        return Err(Error::errno_with_msg(ELOOP, "when expanding shebang"));
    }

    //	/* Push argv[] only on demand.  */
    //	if (argv != NULL) {
    //		status = push_array_of_xpointers(argv, SYSARG_2);
    //		if (status < 0)
    //			return status;
    //	}
    //
    //	return (has_shebang ? 1 : 0);

    Ok(result_host_path.unwrap())
}

//TODO: remove this function
/// Translate a guest path and checks that it's executable.
pub fn translate_and_check_exec(fs: &FileSystem, guest_path: &Path) -> Result<PathBuf> {
    let host_path = fs.translate_path(guest_path, true)?;

    fs.is_path_executable(&host_path)?;

    Ok(host_path)
}

/// Extract into `user_path` and `argument` the shebang from @host_path.
/// This function returns -errno if an error occured, 1 if a shebang
/// was found and extracted, otherwise 0.
///
/// Extract from "man 2 execve":
///
///     On Linux, the entire string following the interpreter name is
///     passed as a *single* argument to the interpreter, and this
///     string can include white space.
//const char *host_path, char user_path[PATH_MAX], char
// argument[BINPRM_BUF_SIZE]
fn extract(host_path: &Path) -> Result<Option<PathBuf>> {
    let mut bytes = BufReader::new(File::open(host_path)?).bytes();
    match (bytes.next(), bytes.next()) {
        (Some(Err(err)), _) | (_, Some(Err(err))) => return Err(Error::from(err)),
        (Some(Ok(b'#')), Some(Ok(b'!'))) => {}
        _ => return Ok(None),
    }
    let first_line = bytes
        .take_while(|c| match c {
            Ok(b'\n') => true,
            _ => false,
        })
        .collect::<std::result::Result<Vec<u8>, _>>()?;
    let first_line = first_line.trim();

    let path = &first_line[..first_line
        .iter()
        .position(|c| c.is_ascii_whitespace())
        .unwrap_or(first_line.len())];

    if path.is_empty() {
        return Err(Error::errno_with_msg(
            ENOEXEC,
            format!("Empty shebang detected, host_path: {:?}", host_path),
        ));
    }
    // NOTE: this unwrap may fail on non-UNIX systems (a.k.a Windows)
    // where paths may not be arbitrary bytes
    let arg = first_line[path.len()..].trim();

    let mut argv = PathBuf::from(path.as_bstr().to_path().unwrap());
    argv.push(arg.as_bstr().to_path().unwrap()); // FIXME: why append arg here?
    Ok(Some(argv))
    //
    //	/* Skip leading spaces. */
    //	do {
    //		status = read(fd, &tmp, sizeof(char));
    //		if (status < 0) {
    //			status = -errno;
    //			goto end;
    //		}
    //		if ((size_t) status < sizeof(char)) { /* EOF */
    //			status = -ENOEXEC;
    //			goto end;
    //		}
    //
    //		current_length++;
    //	} while ((tmp == ' ' || tmp == '\t') && current_length <
    // BINPRM_BUF_SIZE);
    //
    //	/* Slurp the interpreter path until the first space or end-of-line. */
    //	for (i = 0; current_length < BINPRM_BUF_SIZE; current_length++, i++) {
    //		switch (tmp) {
    //		case ' ':
    //		case '\t':
    //			/* Remove spaces in between the interpreter
    // 			 * and the hypothetical argument. */
    //			user_path[i] = '\0';
    //			break;
    //
    //		case '\n':
    //		case '\r':
    //			/* There is no argument. */
    //			user_path[i] = '\0';
    //			argument[0] = '\0';
    //			status = 1;
    //			goto end;
    //
    //		default:
    //			/* There is an argument if the previous
    // 			 * character in user_path[] is '\0'. */
    //			if (i > 1 && user_path[i - 1] == '\0')
    //				goto argument;
    //			else
    //				user_path[i] = tmp;
    //			break;
    //		}
    //
    //		status = read(fd, &tmp, sizeof(char));
    //		if (status < 0) {
    //			status = -errno;
    //			goto end;
    //		}
    //		if ((size_t) status < sizeof(char)) { /* EOF */
    //			user_path[i] = '\0';
    //			argument[0] = '\0';
    //			status = 1;
    //			goto end;
    //		}
    //	}
    //
    //	/* The interpreter path is too long, truncate it. */
    //	user_path[i] = '\0';
    //	argument[0] = '\0';
    //	status = 1;
    //	goto end;
    //
    //argument:
    //
    //	/* Slurp the argument until the end-of-line. */
    //	for (i = 0; current_length < BINPRM_BUF_SIZE; current_length++, i++) {
    //		switch (tmp) {
    //		case '\n':
    //		case '\r':
    //			argument[i] = '\0';
    //
    //			/* Remove trailing spaces. */
    //			for (i--; i > 0 && (argument[i] == ' ' || argument[i] == '\t'); i--)
    //				argument[i] = '\0';
    //
    //			status = 1;
    //			goto end;
    //
    //		default:
    //			argument[i] = tmp;
    //			break;
    //		}
    //
    //		status = read(fd, &tmp, sizeof(char));
    //		if (status < 0) {
    //			status = -errno;
    //			goto end;
    //		}
    //		if ((size_t) status < sizeof(char)) { /* EOF */
    //			argument[0] = '\0';
    //			status = 1;
    //			goto end;
    //		}
    //	}
    //
    //	/* The argument is too long, truncate it. */
    //	argument[i] = '\0';
    //	status = 1;
    //
    //end:
    //	close(fd);
    //
    //	/* Did an error occur or isn't a script? */
    //	if (status <= 0)
    //		return status;
    //
    //	return 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{filesystem::FileSystem, utils::tests::get_test_rootfs};
    use std::path::PathBuf;

    #[test]
    fn test_extract_shebang_not_script() {
        let rootfs_path = get_test_rootfs();

        // it should detect that `/bin/sleep` is not a script
        assert_eq!(Ok(None), extract(&rootfs_path.join("bin/sleep")));
    }

    // TODO: test shebang expand not contains shebang
    #[test]
    fn test_expand_shebang_no_exec_permission() {
        let rootfs_path = get_test_rootfs();

        let fs = FileSystem::with_root(&rootfs_path);

        // it should detect that `/etc/hostname` is not executable
        assert_eq!(
            Err(Error::errno(Errno::EACCES)),
            expand(&fs, &PathBuf::from("/etc/passwd"))
        );
    }
}
