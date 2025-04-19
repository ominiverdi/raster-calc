// src/processing/indices/mod.rs
pub mod ndi;
pub mod evi;
pub mod savi;
pub mod ndwi;
pub mod ndsi;
pub mod bsi;

// Re-export indices
pub use ndi::NDI;
pub use evi::EVI;
pub use savi::SAVI;
pub use ndwi::NDWI;
pub use ndsi::NDSI;
pub use bsi::BSI;