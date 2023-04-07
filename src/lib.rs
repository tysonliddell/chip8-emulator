// TODO: Do not allow unused code. This is here to keep pre-commit hooks happy
// while developing.
#![allow(unused)]

#[cfg(test)]
#[macro_use]
mod test_utils;

// Modules
mod error;
mod interpreter;
pub mod memory;

// Reexports
pub use error::Error;

// Private helpers
type Result<T> = std::result::Result<T, Error>;

#[cfg(debug_assertions)]
mod debug;

// #[cfg(debug_assertions)]
// macro_rules! debug {
//     ($x:expr) => { dbg!($x) }
// }

// #[cfg(not(debug_assertions))]
// macro_rules! debug {
//     ($x:expr) => { }
// }
