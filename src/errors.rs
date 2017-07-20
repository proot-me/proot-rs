use std::{string, error, result, fmt};
use std::io::{Error as IOError, ErrorKind as IOErrorKind, CharsError};
use nix::errno;
use nix::Error as NixError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Sys(errno::Errno),
    InvalidPath,
    InvalidUtf8,
    IOError(IOErrorKind),
    UnsupportedOperation,
}

impl Error {
    pub fn from_errno(errno: errno::Errno) -> Error {
        Error::Sys(errno)
    }

    pub fn invalid_argument() -> Error {
        Error::Sys(errno::EINVAL)
    }

    pub fn name_too_long() -> Error {
        Error::Sys(errno::ENAMETOOLONG)
    }

    pub fn no_such_file_or_dir() -> Error {
        Error::Sys(errno::ENOENT)
    }

    pub fn not_a_directory() -> Error {
        Error::Sys(errno::ENOTDIR)
    }

    pub fn too_many_symlinks() -> Error {
        Error::Sys(errno::ELOOP)
    }

    pub fn cant_exec() -> Error {
        Error::Sys(errno::ENOEXEC)
    }

    pub fn not_supported() -> Error {
        Error::Sys(errno::EOPNOTSUPP)
    }
}

impl From<errno::Errno> for Error {
    fn from(errno: errno::Errno) -> Error {
        Error::from_errno(errno)
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
            Some(errno) => Error::Sys(errno::Errno::from_i32(errno)),
            // if not successful, we keep the IOError to retain the context of the error
            None => Error::IOError(io_error.kind()),
        }
    }
}

impl From<NixError> for Error {
    fn from(nix_error: NixError) -> Error {
        match nix_error {
            NixError::Sys(errno) => Error::Sys(errno),
            NixError::InvalidPath => Error::InvalidPath,
            NixError::InvalidUtf8 => Error::InvalidUtf8,
            NixError::UnsupportedOperation => Error::UnsupportedOperation,
        }
    }
}

impl From<CharsError> for Error {
    fn from(chars_error: CharsError) -> Error {
        match chars_error {
            CharsError::NotUtf8 => Error::InvalidUtf8,
            CharsError::Other(error) => error.into(),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::InvalidPath => "Invalid path",
            &Error::InvalidUtf8 => "Invalid UTF-8 string",
            &Error::Sys(ref errno) => errno.desc(),
            &Error::IOError(_) => "IO Error",
            &Error::UnsupportedOperation => "Unsupported Operation"
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::InvalidPath => write!(f, "Invalid path"),
            &Error::InvalidUtf8 => write!(f, "Invalid UTF-8 string"),
            &Error::Sys(errno) => write!(f, "{:?}: {}", errno, errno.desc()),
            &Error::IOError(io_error_kind) => write!(f, "IO Error: {:?}", io_error_kind),
            &Error::UnsupportedOperation => write!(f, "Unsupported Operation")
        }
    }
}
