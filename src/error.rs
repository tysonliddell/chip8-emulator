use std::fmt;
// use std::io;

/// The error type used throughout this library.
#[derive(Debug, PartialEq)]
pub enum Error {
    // Io(io::Error),
    EmptyChip8Program,
    Chip8ProgramTooLarge(usize),
    RamOverflow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Error::Io(err) => write!(f, "IO error: {}", err),
            Error::EmptyChip8Program => write!(f, "CHIP-8 program is empty!"),
            Error::Chip8ProgramTooLarge(size) => {
                write!(f, "CHIP-8 program with size {} bytes is too large!", size)
            }
            Error::RamOverflow => write!(f, "Operation would cause a write beyond the end of RAM."),
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
