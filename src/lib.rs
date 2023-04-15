#[cfg(test)]
#[macro_use]
mod test_utils;

// Modules
pub mod emulator;
mod error;
mod font;
mod interpreter;
pub mod memory;
pub mod peripherals;
mod rng;

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
