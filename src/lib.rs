// src/lib.rs
pub mod cli;
pub mod io;
pub mod utils;



// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");