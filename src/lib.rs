// src/lib.rs
pub mod cli;
pub mod indices;
pub mod io;
pub mod utils;

// Re-export main functionality for easier library usage
pub use indices::ndi::calculate_ndi;

// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");