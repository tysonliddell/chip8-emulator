//! Roms hold CHIP-8 program data.
use std::fmt::Debug;

use crate::memory::{ROM_LAST_ADDRESS, ROM_START_ADDRESS};
use crate::{Error, Result};

/// The CHIP-8 has 4096 bytes of memory, but the first 512 bytes are reserved.
pub const MAX_ROM_SIZE: usize = ROM_LAST_ADDRESS - ROM_START_ADDRESS + 1;

/// A program to be executed on the CHIP-8.
pub struct Rom {
    name: String,
    data: Vec<u8>,
}

impl Rom {
    /// Create a `Rom` from copied bytes.
    ///
    /// # Errors
    /// Can return [`Error::EmptyRom`] or [`Error::RomTooLarge`].
    pub fn from_bytes(name: &str, rom_bytes: &[u8]) -> Result<Self> {
        if rom_bytes.is_empty() {
            return Err(Error::EmptyRom);
        } else if rom_bytes.len() > MAX_ROM_SIZE {
            return Err(Error::RomTooLarge(rom_bytes.len()));
        }

        Ok(Rom {
            name: name.to_string(),
            data: rom_bytes.to_vec(),
        })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Debug for Rom {
    /// Returns the rom name and up to the first 10 bytes of the rom.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.bytes();
        write!(f, "{}: {:?}", self.name, &bytes[..10.min(bytes.len())])
    }
}

#[cfg(test)]
mod tests {
    use super::Rom;
    use crate::{rom::MAX_ROM_SIZE, Error};

    #[test]
    fn zero_bytes_rom() {
        let res = Rom::from_bytes("test_name", &[]);
        assert_eq!(res.unwrap_err(), Error::EmptyRom);
    }

    #[test]
    fn max_size_rom() {
        // Since the first 512 bytes are reserved, the expected max
        // rom size is 4096 - 512.
        assert_eq!(MAX_ROM_SIZE, 4096 - 512);

        let max_bytes = [0u8; MAX_ROM_SIZE];
        let res = Rom::from_bytes("test_name", &max_bytes);
        println!("{:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn rom_too_large() {
        const TOO_LARGE: usize = MAX_ROM_SIZE + 1;
        let one_too_many_bytes = [0u8; TOO_LARGE];
        let res = Rom::from_bytes("test_name", &one_too_many_bytes);
        assert_eq!(res.unwrap_err(), Error::RomTooLarge(TOO_LARGE));
    }
}
