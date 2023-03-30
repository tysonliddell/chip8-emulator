// Modules
mod error;
mod memory;
pub mod rom;

// Reexports
pub use error::Error;

// Private helpers
type Result<T> = std::result::Result<T, Error>;
