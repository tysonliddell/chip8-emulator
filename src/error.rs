use std::fmt;
// use std::io;

/// The error type used throughout this library.
#[derive(Debug, PartialEq)]
pub enum Error {
    // Io(io::Error),
    EmptyRom,
    RomTooLarge(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Error::Io(err) => write!(f, "IO error: {}", err),
            Error::EmptyRom => write!(f, "Rom is empty!"),
            Error::RomTooLarge(size) => write!(f, "Rom with size {} bytes is too large!", size),
        }
    }
}

impl std::error::Error for Error {
    // Don't implement `description` or `cause` trait methods as they are deprecated.

    // fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    //     match self {
    //         Error::Io(err) => Some(err),
    //         _ => None,
    //     }
    // }
}

// impl From<io::Error> for Error {
//     fn from(err: io::Error) -> Self {
//         Self::Io(err)
//     }
// }
