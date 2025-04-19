// src/processing/indices/mod.rs
pub mod ndi;
pub mod evi;
pub mod savi;

// Re-export indices
pub use ndi::NDI;
pub use evi::EVI;
pub use savi::SAVI;