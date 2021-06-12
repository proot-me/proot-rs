pub use nix::errno::Errno::{self, *};
use nix::Error as NixError;
use std::io::Error as IOError;
use std::{
    fmt::{self, Display},
    result,
};
pub type Result<T> = result::Result<T, Error>;

/// This struct is an abstraction of exceptions encountered in the code. It is
/// inspired by [`anyhow`]. All type `E` which implements`std::error::Error` can
/// be converted to this `Error`. In addition, it contains an `errno` field,
/// which is useful in scenarios where errno value needs to be returned.
///
/// [`anyhow`]: https://docs.rs/anyhow/1.0.40/anyhow/

pub struct Error {
    errno: Errno,
    msg: Option<Box<dyn Display + Send + Sync + 'static>>,
    source: Option<Box<dyn std::error::Error>>,
}

#[allow(dead_code)]
impl Error {
    /// Create an Error with a unknown errno
    pub fn unknown() -> Self {
        Error::errno(Errno::UnknownErrno)
    }

    /// Create an Error with the specific errno
    pub fn errno(errno: Errno) -> Self {
        Error {
            errno: errno,
            msg: None,
            source: None,
        }
    }

    /// Create an Error with the specific message
    pub fn msg<M>(msg: M) -> Self
    where
        M: Display + Send + Sync + 'static,
    {
        Error::errno_with_msg(Errno::UnknownErrno, msg)
    }

    /// Create an Error with the specific errno and message
    pub fn errno_with_msg<M>(errno: Errno, msg: M) -> Self
    where
        M: Display + Send + Sync + 'static,
    {
        Error {
            errno: errno,
            msg: Some(Box::new(msg)),
            source: None,
        }
    }

    /// Set errno of self to a specific errno, and return this Error.
    pub fn with_errno(mut self, errno: Errno) -> Self {
        self.errno = errno;
        self
    }

    /// Set message of self to a specific message, and return this Error.
    pub fn with_msg<M>(mut self, msg: M) -> Self
    where
        M: Display + Send + Sync + 'static,
    {
        self.msg = Some(Box::new(msg));
        self
    }

    /// Get errno of this Error. If errno is not set, the default value is
    /// `UnknownErrno`.
    pub fn get_errno(&self) -> Errno {
        self.errno
    }
}

#[allow(dead_code)]
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error with {}({})", self.errno, self.errno as i32)?;

        if let Some(msg) = &self.msg {
            write!(f, ", msg: {}", msg)?;
        }
        if let Some(source) = &self.source {
            write!(f, ", source: {}", source)?;
        }
        Ok(())
    }
}

#[allow(dead_code)]
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Error");
        d.field("errno", &self.errno);
        match self.msg.as_ref() {
            Some(msg) => d.field("msg", &Some(format_args!("{}", msg))),
            None => d.field("msg", &Option::<()>::None),
        };
        d.field("source", &self.source).finish()
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.errno == other.errno
    }
}

impl From<Errno> for Error {
    fn from(errno: Errno) -> Error {
        Error::errno(errno)
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error + 'static,
{
    default fn from(error: E) -> Self {
        Error {
            errno: UnknownErrno,
            msg: None,
            source: Some(Box::new(error)),
        }
    }
}

impl From<IOError> for Error {
    fn from(error: IOError) -> Error {
        Error {
            errno: match error.raw_os_error() {
                // we try to convert it to an errno
                Some(errno) => Errno::from_i32(errno),
                None => Errno::UnknownErrno,
            },
            msg: None,
            source: Some(Box::new(error)),
        }
    }
}

impl From<NixError> for Error {
    fn from(error: NixError) -> Error {
        Error {
            errno: match error {
                NixError::Sys(errno) => errno,
                _ => Errno::UnknownErrno,
            },
            msg: None,
            source: Some(Box::new(error)),
        }
    }
}

/// This trait is something like [`anyhow::Context`], which provide
/// `with_context()` and `context()` function to attach a message to
/// `Result<T,E>`, In addition, it also allows appending an `errno` value.
///
/// [`anyhow::Context`]: https://docs.rs/anyhow/1.0.40/anyhow/trait.Context.html
#[allow(dead_code)]
pub trait WithContext<T> {
    fn errno(self, errno: Errno) -> Result<T>;

    fn context<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

#[allow(dead_code)]
impl<T, E> WithContext<T> for result::Result<T, E>
where
    Error: From<E>,
{
    default fn errno(self, errno: Errno) -> Result<T> {
        self.map_err(|error| Into::<Error>::into(error).with_errno(errno))
    }

    default fn context<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|error| Into::<Error>::into(error).with_msg(context))
    }

    default fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|error| Into::<Error>::into(error).with_msg(f()))
    }
}
