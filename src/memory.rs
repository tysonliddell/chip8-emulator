//! The CHIP-8 that ran on the COSMAC VIP had between 2048 and 4096 bytes of
//! memory, divided into pages of 256 bytes each.
//!
//! # Memory map
//! The diagram below shows the memory layout, where `CAPACITY` is `0x1000` for
//! the 4K system and `0x0800` for the 2K system.
//!
//! ```text
//! +-----------------------------------------+ 0x000
//! | CHIP-8 language interpreter (512 bytes) |
//! +-----------------------------------------+ 0x200
//! | User program/rom (1184 or 3232 bytes)   |
//! +-----------------------------------------+ CAPACITY - 256 - 48 - 48
//! | CHIP-8 stack (48 bytes)                 |
//! +-----------------------------------------+ CAPACITY - 256 - 48
//! | CHIP-8 interpreter work area (48 bytes) |
//! | Last 16 bytes contain V0-VF registers   |
//! +-----------------------------------------+ CAPACITY - 256
//! | Display refresh (256 bytes)             |
//! +-----------------------------------------+ CAPACITY
//! | Operating system ROM                    |
//! +-----------------------------------------+
//!
//! 4K memory map
//! +-----------------------------------------+ 0x0000
//! | CHIP-8 language interpreter (512 bytes) |
//! +-----------------------------------------+ 0x0200
//! | User program/rom (1184 or 3232 bytes)   |
//! +-----------------------------------------+ 0x0EA0
//! | CHIP-8 stack (48 bytes)                 |
//! +-----------------------------------------+ 0x0ED0
//! | CHIP-8 interpreter work area (48 bytes) |
//! | 0x0EF0 - 0x0EFF contain V0-VF registers |
//! +-----------------------------------------+ 0x0F00
//! | Display refresh (256 bytes)             |
//! +-----------------------------------------+ 0x1000
//! | Operating system ROM                    |
//! +-----------------------------------------+
//!
//! 2K memory map
//! +-----------------------------------------+ 0x0000
//! | CHIP-8 language interpreter (512 bytes) |
//! +-----------------------------------------+ 0x0200
//! | User program/rom (1184 or 3232 bytes)   |
//! +-----------------------------------------+ 0x06A0
//! | CHIP-8 stack (48 bytes)                 |
//! +-----------------------------------------+ 0x06D0
//! | CHIP-8 interpreter work area (48 bytes) |
//! | 0x06F0 - 0x06FF contain V0-VF registers |
//! +-----------------------------------------+ 0x0700
//! | Display refresh (256 bytes)             |
//! +-----------------------------------------+ 0x0800
//! | Operating system ROM                    |
//! +-----------------------------------------+
//! ```
//!
//! # CHIP-8 memory organization on the COSMAC VIP
//! In normal operation, the COSMAC VIP natively runs CDP1802 machine language
//! instructions starting at address `0x0000`. But first, to load a program into
//! RAM, the COSMAC operating system ROM is booted (by holding the `C` key on
//! the hex keyboard during startup). This allows each byte of the program to
//! be written to memory by hand, one byte at a time, using the hex keyboard or
//! read in from cassette tape.
//!
//! To run a CHIP-8 program on the COSMAC VIP, the CHIP-8 language interpreter,
//! written in CDP1802 machine language, first needs to be loaded into
//! addresses `0x0000` - `0x01FF`. The CHIP-8 program itself then needs to be
//! loaded into memory, starting at address `0x0200`.
//!
//! The CHIP-8 stack is used by the CHIP-8 interpreter to store the subroutine
//! return addresses.
//!
//! The CHIP-8 interpreter work area contains the CHIP-8 "registers" and is used
//! by the interpreter (presumably to emulate the CHIP-8 fetch-decode-execute
//! cycle).
//!
//! The last page of RAM is used by the CHIP-8 interpreter for display refresh.

use crate::{Error, Result};
const _SMALL_MEMORY_SIZE: usize = 0x0800; // The 2K system
const _LARGE_MEMORY_SIZE: usize = 0x1000; // The beefier 4K system
pub const MEMORY_SIZE: usize = _LARGE_MEMORY_SIZE;

pub const _MEMORY_START_ADDRESS: usize = 0x000;
pub const PROGRAM_START_ADDRESS: usize = 0x200;
pub const STACK_START_ADDRESS: usize = 0xEA0;
pub const _INTERPRETER_START_ADDRESS: usize = 0x0ED0;
pub const _DISPLAY_REFRESH_START_ADDRESS: usize = 0xF00;

pub const PROGRAM_LAST_ADDRESS: usize = STACK_START_ADDRESS - 1;
pub const PROGRAM_MAX_SIZE: usize = PROGRAM_LAST_ADDRESS - PROGRAM_START_ADDRESS + 1;

/// Main memory used by the CHIP-8 interpreter. Follows COSMAC VIP layout.
pub struct CosmacRAM {
    data: [u8; MEMORY_SIZE],
}

impl CosmacRAM {
    /// Create 4K of COSMAC RAM.
    pub fn new() -> Self {
        Self {
            data: [0; MEMORY_SIZE],
        }
    }

    /// A read-only view of the data in RAM.
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    /// Loads a sequence of bytes into memory starting at the address given by
    /// `memory_offset`.
    ///
    /// # Example
    /// ```
    /// # use chip8_emulator::memory::CosmacRAM;
    /// // Load 4 bytes into the beginning of the third page of memory (pages
    /// // are 256 bytes in size).
    /// let bytes = [0x11, 0x22, 0x33, 0x44];
    /// let mut ram = CosmacRAM::new();
    /// assert!(ram.load_bytes(&bytes, 0x0300).is_ok());
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::RamOverflow`] if bytes cannot fit into RAM at the given offset.
    /// When this occurs no change is made to the RAM.
    ///
    /// ```
    /// // Attempt to load bytes where they cannot fit into RAM.
    /// # use chip8_emulator::memory::CosmacRAM;
    /// # use chip8_emulator::memory::MEMORY_SIZE;
    /// let program = [0x00, 0x00];
    /// let mut ram = CosmacRAM::new();
    /// assert!(ram.load_bytes(&program, MEMORY_SIZE).is_err());
    /// assert!(ram.load_bytes(&program, MEMORY_SIZE-1).is_err());
    /// assert!(ram.load_bytes(&program, MEMORY_SIZE-2).is_ok());
    /// ```
    pub fn load_bytes(&mut self, bytes: &[u8], ram_offset: usize) -> Result<()> {
        let (target_start, target_end) = (ram_offset, ram_offset + bytes.len());
        if target_end > MEMORY_SIZE {
            return Err(Error::RamOverflow);
        }

        let ram_target = &mut self.data[target_start..target_end];

        ram_target.copy_from_slice(bytes);
        Ok(())
    }

    /// Load a CHIP-8 program, given in bytes, into the pages of memory expected
    /// by a CHIP-8 interpreter.
    ///
    /// # Errors
    /// Can return [`Error::EmptyChip8Program`] or [`Error::Chip8ProgramTooLarge`].
    ///
    /// # Example
    /// ```
    /// # use chip8_emulator::memory::CosmacRAM;
    /// // Load a CHIP-8 program into memory that draws a single pixel at (0,0).
    /// // Program: A300 6080 F055 6000 A300 D001 120C
    /// let program = [
    ///     0xA3, 0x00, 0x60, 0x80, 0xF0, 0x55, 0x60, 0x00, 0xA3, 0x00, 0xD0, 0x01, 0x12, 0x0C
    /// ];
    /// let mut ram = CosmacRAM::new();
    /// assert!(ram.load_chip8_program(&program).is_ok());
    /// ```
    pub fn load_chip8_program(&mut self, chip8_program: &[u8]) -> Result<()> {
        if chip8_program.is_empty() {
            return Err(Error::EmptyChip8Program);
        } else if PROGRAM_START_ADDRESS + chip8_program.len() - 1 > PROGRAM_LAST_ADDRESS {
            return Err(Error::Chip8ProgramTooLarge(chip8_program.len()));
        }

        self.load_bytes(chip8_program, PROGRAM_START_ADDRESS)
            .expect("Memory bounds already checked.");
        Ok(())
    }
}

impl Default for CosmacRAM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use crate::Error;

    use super::{
        CosmacRAM, MEMORY_SIZE, PROGRAM_LAST_ADDRESS, PROGRAM_MAX_SIZE, PROGRAM_START_ADDRESS,
        STACK_START_ADDRESS, _DISPLAY_REFRESH_START_ADDRESS, _INTERPRETER_START_ADDRESS,
        _MEMORY_START_ADDRESS,
    };

    /// Convert a slice of `u16` to a vector of `u8` populated in COSMAC byte order.
    /// This is useful when moving an array of 16-bit CHIP-8 instruction literals
    /// (stored as `u16`s) into the 8-bit [`CosmacRAM`], which is big endian.
    fn cosmac_bytes_from_u16(data: &[u16]) -> Vec<u8> {
        data.iter().copied().flat_map(u16::to_be_bytes).collect()
    }

    #[test]
    fn memory_boundaries() {
        assert_eq!(MEMORY_SIZE, 4096);
        assert_eq!(MEMORY_SIZE - _DISPLAY_REFRESH_START_ADDRESS, 256);
        assert_eq!(
            _DISPLAY_REFRESH_START_ADDRESS - _INTERPRETER_START_ADDRESS,
            48
        );
        assert_eq!(_INTERPRETER_START_ADDRESS - STACK_START_ADDRESS, 48);

        // CHIP-8 programs are allowed to use an extra 2048 bytes when using 4K of RAM instead of 2K.
        assert_eq!(STACK_START_ADDRESS - PROGRAM_START_ADDRESS, 1184 + 2048);
        assert_eq!(PROGRAM_LAST_ADDRESS, STACK_START_ADDRESS - 1);
        assert_eq!(PROGRAM_START_ADDRESS - _MEMORY_START_ADDRESS, 512);
    }

    #[test]
    fn ram_overflow() {
        let program = [0x00, 0x00];
        let mut ram = CosmacRAM::new();
        assert!(ram.load_bytes(&program, MEMORY_SIZE).is_err());
        assert!(ram.load_bytes(&program, MEMORY_SIZE - 1).is_err());
        assert!(ram.load_bytes(&program, MEMORY_SIZE - 2).is_ok());
    }

    #[test]
    fn load_into_ram() {
        let program = [0xA300u16, 0x6080, 0xF055, 0x6000, 0xA300, 0xD001, 0x120C];
        let program = cosmac_bytes_from_u16(&program);

        let mut ram = CosmacRAM::new();
        assert!(ram.load_bytes(&program, 0).is_ok());

        assert_eq!(ram.data[0], 0xA3, "TEST!");
        assert_eq!(ram.data[1], 0x00, "TEST!");
    }

    #[test]
    fn chip8_no_data() {
        let mut ram = CosmacRAM::new();
        assert_eq!(
            ram.load_chip8_program(&[]).unwrap_err(),
            Error::EmptyChip8Program
        );
    }

    #[test]
    fn chip8_program_too_big() {
        let program_too_big = [0x00; PROGRAM_MAX_SIZE + 1];
        let program_max_size = [0x00; PROGRAM_MAX_SIZE];
        let mut ram = CosmacRAM::new();

        assert_eq!(
            ram.load_chip8_program(&program_too_big).unwrap_err(),
            Error::Chip8ProgramTooLarge(PROGRAM_MAX_SIZE + 1)
        );
        assert!(
            ram.load_chip8_program(&program_max_size).is_ok(),
            "A CHIP-8 program of max size should be accepted into RAM."
        );
    }

    #[test]
    fn load_bytes_does_not_trash_other_memory() {
        let original_data = [0x01, 0x02, 0x03, 0x04, 0x05];
        let new_data = [0x00, 0x00];
        let mut ram = CosmacRAM::new();

        ram.load_bytes(&original_data, 0)
            .expect("Loading these bytes should not fail!");
        ram.load_bytes(&new_data, 1)
            .expect("Loading these bytes should not fail!");

        assert_eq!(
            ram.data[..5],
            [0x01, 0x00, 0x00, 0x04, 0x05],
            "Expect only the second 2 bytes to be zeroed"
        );
    }

    #[test]
    fn load_bytes_copies_data() {
        let original_data = [0x01, 0x02, 0x03, 0x04, 0x05];
        let mut ram = CosmacRAM::new();
        ram.load_bytes(&original_data, 0)
            .expect("Loading these bytes should not fail!");

        let mut ram = ram;
        ram.data[0] = 0x42;
        assert_eq!(
            original_data[0], 0x01,
            "Don't expect source data to be modified after copying it into RAM."
        )
    }

    #[test]
    fn u16_to_u8_conversion() {
        let data = [0x1122, 0x3344];
        let bytes = cosmac_bytes_from_u16(&data);
        assert_eq!(bytes, [0x11, 0x22, 0x33, 0x44]);
    }
}
