// TODO: Do not allow unused code. This is here to keep pre-commit hooks happy
// while developing.
#![allow(unused)]

// Modules
mod error;
mod interpreter;
pub mod memory;

// Reexports
pub use error::Error;

// Private helpers
type Result<T> = std::result::Result<T, Error>;
