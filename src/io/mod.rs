// src/io/mod.rs
pub mod reader;
pub mod writer;

pub use reader::read_bands_parallel;
pub use writer::write_raster;