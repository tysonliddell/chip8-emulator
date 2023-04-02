// Modules
mod error;
mod interpreter;
pub mod memory;

// Reexports
pub use error::Error;

// Private helpers
type Result<T> = std::result::Result<T, Error>;
