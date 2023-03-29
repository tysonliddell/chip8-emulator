// Modules
mod error;
pub mod rom;

// Reexports
pub use error::Error;

// Private helpers
type Result<T> = std::result::Result<T, Error>;

mod memory {
    //! The CHIP-8 consists of 4096 bytes of memory with addresses 0x000 to 0xFFF.
    //!
    //! Addresses 0x000 - 0x1FF (inclusive) were traditionally used to store the CHIP-8
    //! interpreter. This emulator does not reside within these first 512 bytes, but like
    //! other modern CHIP-8 emulators, this space is reserved for font data.
    pub const ROM_START_ADDRESS: usize = 0x200;
    pub const ROM_LAST_ADDRESS: usize = 0xFFF;
}
