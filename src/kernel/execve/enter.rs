use errors::{Error, Result};
use filesystem::Translator;
use kernel::execve::load_info::LoadInfo;
use kernel::execve::loader::LoaderFile;
use kernel::execve::shebang;
use nix::errno::Errno;
use process::tracee::Tracee;
use register::{PtraceReader, SysArg1};

pub fn translate(tracee: &mut Tracee, loader: &dyn LoaderFile) -> Result<()> {
    //TODO: implement this part for ptrace translation
    //	if (IS_NOTIFICATION_PTRACED_LOAD_DONE(tracee)) {
    //		/* Syscalls can now be reported to its ptracer.  */
    //		tracee->as_ptracee.ignore_loader_syscalls = false;
    //
    //		/* Cancel this spurious kernel.execve, it was only used as a
    //		 * notification.  */
    //		set_sysnum(tracee, PR_void);
    //		return 0;
    //	}

    let raw_path = tracee.regs.get_sysarg_path(SysArg1)?;
    //TODO: return user path
    let host_path = match shebang::expand(&tracee.fs, &raw_path) {
        Ok(path) => path,
        // The Linux kernel actually returns -EACCES when trying to execute a directory.
        Err(Error::Sys(Errno::EISDIR, _)) => return Err(Error::from(Errno::EACCES)),
        Err(error) => return Err(error),
    };

    //TODO: clear this when raw_path and user_path's implementations are done
    //	/* user_path is modified only if there's an interpreter
    //	 * (ie. for a script or with qemu).  */
    //	if (status == 0 && tracee->qemu == NULL)
    //		TALLOC_FREE(raw_path);

    //	Remember the new value for "/proc/self/exe".  It points to
    //	a canonicalized guest path, hence detranslate_path()
    //	instead of using user_path directly.  */
    if let Ok(maybe_path) = tracee.fs.detranslate_path(&host_path, None) {
        tracee.new_exe = Some(maybe_path.unwrap_or_else(|| host_path.clone()));
    } else {
        tracee.new_exe = None;
    }

    //TODO: implement runner for qemu
    //	if (tracee->qemu != NULL) {
    //		status = expand_runner(tracee, host_path, user_path);
    //		if (status < 0)
    //			return status;
    //	}

    let mut load_info = LoadInfo::from(&tracee.fs, &host_path)?;

    load_info.raw_path = Some(raw_path.clone());
    //TODO: use user_path when implemented
    load_info.user_path = Some(raw_path);
    load_info.host_path = Some(host_path);

    if load_info.interp.is_none() {
        return Err(Error::invalid_argument(
            "when translating enter execve, interp is none",
        ));
    }
    if let Some(ref interp) = load_info.interp {
        if interp.interp.is_some() {
            return Err(Error::invalid_argument(
                "when translating enter execve, an ELF interpreter is supposed to be standalone.",
            ));
        }
    }

    load_info.compute_load_addresses(false)?;

    // Execute the loader instead of the program
    loader.prepare_loader()?;

    // Save the loader path in the register, so that the loader will be executed instead.
    //TODO: uncomment this when execve::exit is ready
    /*
    regs.set_sysarg_path(
        SysArg1,
        loader.get_loader_path(),
        "during enter execve translation, setting new loader path",
    )?;
    */

    //TODO: implemented ptracee translation
    //	/* Mask to its ptracer kernel performed by the loader.  */
    //	tracee->as_ptracee.ignore_loader_syscalls = true;
    //
    //	return 0;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix::unistd::execvp;
    use register::{Current, PtraceReader};
    use sc::nr::{EXECVE, NANOSLEEP};
    use std::ffi::CString;
    use utils::tests::fork_test;

    #[test]
    fn test_execve_translate_enter() {
        let mut at_least_one_translation_occured = false;

        fork_test(
            "/",
            // expecting a normal execution
            0,
            // parent
            |tracee, info_bag| {
                if tracee.regs.get_sys_num(Current) == EXECVE {
                    let dir_path = tracee.regs.get_sysarg_path(SysArg1).unwrap();
                    let file_exists = dir_path.exists();

                    // if the file executed by execve exists, we expect the translation to go well.
                    if file_exists {
                        assert_eq!(Ok(()), translate(tracee, &info_bag.loader));
                        at_least_one_translation_occured = true;
                    }
                    false
                } else if tracee.regs.get_sys_num(Current) == NANOSLEEP {
                    // we expect at least one successful translation to have occurred
                    assert!(at_least_one_translation_occured);

                    // we stop when the NANOSLEEP syscall is detected
                    true
                } else {
                    false
                }
            },
            // child
            || {
                // calling the sleep function, which should call the NANOSLEEP syscall
                execvp(
                    &CString::new("sleep").unwrap(),
                    &[&CString::new(".").unwrap(), &CString::new("0").unwrap()],
                )
                .expect("failed execvp sleep");
            },
        );
    }
}
