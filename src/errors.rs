use nix::errno;
use nix::Error as NixError;
use std::io::{Error as IOError, ErrorKind as IOErrorKind};
use std::{error, fmt, result, string};

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Sys(errno::Errno, &'static str),
    InvalidPath(&'static str),
    InvalidUtf8,
    IOError(IOErrorKind),
    UnsupportedOperation(&'static str),
}

impl Error {
    pub fn get_errno(&self) -> i32 {
        match *self {
            Error::Sys(errno, _) => -(errno as i32),
            _ => 0, //TODO: specify errno for other types of error
        }
    }

    pub fn from_errno(errno: errno::Errno, message: &'static str) -> Error {
        Error::Sys(errno, message)
    }

    pub fn invalid_argument(message: &'static str) -> Error {
        Error::Sys(errno::EINVAL, message)
    }

    pub fn name_too_long(message: &'static str) -> Error {
        Error::Sys(errno::ENAMETOOLONG, message)
    }

    pub fn no_such_file_or_dir(message: &'static str) -> Error {
        Error::Sys(errno::ENOENT, message)
    }

    #[cfg(test)]
    pub fn is_a_directory(message: &'static str) -> Error {
        Error::Sys(errno::EISDIR, message)
    }

    pub fn not_a_directory(message: &'static str) -> Error {
        Error::Sys(errno::ENOTDIR, message)
    }

    pub fn too_many_symlinks(message: &'static str) -> Error {
        Error::Sys(errno::ELOOP, message)
    }

    pub fn cant_exec(message: &'static str) -> Error {
        Error::Sys(errno::ENOEXEC, message)
    }

    pub fn not_supported(message: &'static str) -> Error {
        Error::Sys(errno::EOPNOTSUPP, message)
    }

    pub fn bad_address(message: &'static str) -> Error {
        Error::Sys(errno::EFAULT, message)
    }
}

impl From<errno::Errno> for Error {
    fn from(errno: errno::Errno) -> Error {
        Error::from_errno(errno, "from sys errno")
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(_: string::FromUtf8Error) -> Error {
        Error::InvalidUtf8
    }
}

impl From<IOError> for Error {
    fn from(io_error: IOError) -> Error {
        match io_error.raw_os_error() {
            // we try to convert it to an errno
            Some(errno) => Error::Sys(errno::Errno::from_i32(errno), "from IO error"),
            // if not successful, we keep the IOError to retain the context of the error
            None => Error::IOError(io_error.kind()),
        }
    }
}

impl From<NixError> for Error {
    fn from(nix_error: NixError) -> Error {
        match nix_error {
            NixError::Sys(errno) => Error::Sys(errno, "from Nix error"),
            NixError::InvalidPath => Error::InvalidPath("from Nix error"),
            NixError::InvalidUtf8 => Error::InvalidUtf8,
            NixError::UnsupportedOperation => Error::UnsupportedOperation("from Nix error"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::InvalidPath(_) => "Invalid path",
            &Error::InvalidUtf8 => "Invalid UTF-8 string",
            &Error::Sys(ref errno, _) => errno.desc(),
            &Error::IOError(_) => "IO Error",
            &Error::UnsupportedOperation(_) => "Unsupported Operation",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::InvalidPath(message) => write!(f, "Invalid path ({})", message),
            &Error::InvalidUtf8 => write!(f, "Invalid UTF-8 string"),
            &Error::Sys(errno, message) => write!(f, "{:?}: {} ({})", errno, errno.desc(), message),
            &Error::IOError(io_error_kind) => write!(f, "IO Error: {:?}", io_error_kind),
            &Error::UnsupportedOperation(message) => {
                write!(f, "Unsupported Operation ({})", message)
            }
        }
    }
}
