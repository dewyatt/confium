use std::convert::From;

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! error_codes {
    (
        $(
            ($code:literal, $codename:ident, $kindname:ident, $($errkind:tt)+);
        )+
    ) => {
        #[allow(non_camel_case_types)]
        #[repr(u32)]
        pub enum ErrorCode {
        $(
            $codename = $code,
        )+
        }

        #[derive(Debug)]
        pub enum ErrorKind {
        $(
            $kindname $($errkind)+,
        )+
        }

        impl ErrorKind {
            fn code(&self) -> ErrorCode {
                match self {
                    $(
                        ErrorKind::$kindname { .. } => ErrorCode::$codename,
                    )+
                }
            }
        }
    };
}

error_codes! {
    (1, UNKNOWN, Unknown, {} );
    (2, NULL_POINTER, NullPointer, {});
    (3, IO_ERROR, Io,
        {
            source: ::std::io::Error,
            path: Option<String>,
        }
    );
    (4, INVALID_HEX_DIGIT, InvalidHexDigit, (char));
    (5, INVALID_UTF8, InvalidUTF8, {});
    (6, INVALID_FORMAT, InvalidFormat, {});
    (7, OVERFLOW, Overflow, {});
    (8, PLUGIN_LOAD_ERROR, PluginLoadError, {});
    (9, INITIALIZATION_FAILURE, InitializationFailure, {});
    (10, INVALID_CONFIG, InvalidConfig, {linenum: Option<usize>});
    (11, EXPECTED_TOKEN, ExpectedToken, (char));
}

#[derive(thiserror::Error, Debug)]
pub struct Error {
    pub kind: ErrorKind,
    #[source]
    pub source: Option<Box<dyn std::error::Error>>,
    #[backtrace]
    pub backtrace: Option<std::backtrace::Backtrace>,
}

impl Default for Error {
    fn default() -> Self {
        Error {
            kind: ErrorKind::Unknown {},
            source: None,
            backtrace: None,
        }
    }
}

/*
impl Drop for Error {
    fn drop(&mut self) {
        //println!("Dropping Error {:p}", self);
    }
}
*/

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn code(&self) -> ErrorCode {
        self.kind.code()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorKind::Unknown {} => write!(f, "Unknown error")?,
            ErrorKind::InvalidUTF8 {} => write!(f, "Invalid UTF-8")?,
            ErrorKind::NullPointer {} => write!(f, "Null pointer")?,
            ErrorKind::Io { source, path } => {
                if let Some(path) = path {
                    write!(f, "IO error: '{}': {}", path, source)?
                } else {
                    write!(f, "IO error: {}", source)?
                }
            }
            ErrorKind::InvalidFormat {} => write!(f, "Invalid format")?,
            ErrorKind::InvalidHexDigit(c) => write!(f, "Invalid hex digit: '{}'", c)?,
            ErrorKind::Overflow {} => write!(f, "Overfow")?,
            ErrorKind::PluginLoadError {} => write!(f, "Plugin load error")?,
            ErrorKind::ExpectedToken(c) => write!(f, "Expected '{}'", c)?,
            ErrorKind::InitializationFailure {} => write!(f, "Initialization failure")?,
            ErrorKind::InvalidConfig { linenum } => match linenum {
                Some(linenum) => write!(f, "Invalid config (line {})", linenum)?,
                None => write!(f, "Invalid config")?,
            },
        }
        Ok(())
    }
}

impl From<Error> for u32 {
    #[inline]
    fn from(err: Error) -> u32 {
        err.code() as u32
    }
}

impl From<&Error> for u32 {
    #[inline]
    fn from(err: &Error) -> u32 {
        err.code() as u32
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    /*
    fn test(i: i8) -> Result<(), Error> {
        match i {
            0 => Ok(()),
            1 => Err(ErrorKind::NullPointer),
            _ => Err(Error::Unknown),
        }
    }

    #[test]
    fn test_success() {
        assert_eq!(test(0), Ok(()));
    }

    #[test]
    fn test_nullpointer() {
        assert_eq!(test(1), Err(Error::NullPointer));
    }

    #[test]
    fn test_unknown() {
        assert_eq!(test(2), Err(Error::Unknown));
        assert_eq!(test(100), Err(Error::Unknown));
    }

    #[test]
    fn test_errcode() {
        assert_eq!(test(1).err().unwrap().0 as u32, 2);
        assert_eq!(test(100).err().unwrap().0 as u32, 1);
        assert_eq!(u32::from(test(100).err().unwrap()), 1);
    }
    */
}
