// src/processing/indices/mod.rs
pub mod ndi;
pub mod evi;
pub mod savi;
pub mod ndwi;
pub mod ndsi;
pub mod bsi;
pub mod msavi2;
pub mod osavi;

// Re-export indices
pub use ndi::NDI;
pub use evi::EVI;
pub use savi::SAVI;
pub use ndwi::NDWI;
pub use ndsi::NDSI;
pub use bsi::BSI;
pub use msavi2::MSAVI2;
pub use osavi::OSAVI;