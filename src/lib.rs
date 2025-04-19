// src/lib.rs
pub mod cli;
pub mod utils;
pub mod processing;

// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");