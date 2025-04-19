// src/processing/indices/ndwi.rs
use crate::utils::gdal_ext::TypedBuffer;
use crate::processing::parallel::IndexCalculator;
use rayon::prelude::*;

/// Normalized Difference Water Index (NDWI) calculator
pub struct NDWI {
    green_index: usize,
    nir_index: usize,
    name: String,
}

impl NDWI {
    pub fn new(green_index: usize, nir_index: usize, name: Option<String>) -> Self {
        Self {
            green_index,
            nir_index,
            name: name.unwrap_or_else(|| "NDWI".to_string()),
        }
    }
}

impl IndexCalculator for NDWI {
    fn calculate(&self, inputs: &[TypedBuffer]) -> TypedBuffer {
        // Extract input bands
        let green = &inputs[self.green_index];
        let nir = &inputs[self.nir_index];
        
        // Handle different input types (focusing on f32 for now)
        match (green, nir) {
            (TypedBuffer::F32(green_data), TypedBuffer::F32(nir_data)) => {
                let shape = green_data.shape();
                let green_band = green_data.data();
                let nir_band = nir_data.data();
                
                // Preallocate result buffer
                let mut result_data = vec![0.0f32; shape.0 * shape.1];
                
                // Calculate NDWI in parallel (GREEN - NIR) / (GREEN + NIR)
                result_data.par_iter_mut().enumerate().for_each(|(i, result)| {
                    let green_val = green_band[i];
                    let nir_val = nir_band[i];
                    
                    *result = if green_val + nir_val > 0.0 {
                        (green_val - nir_val) / (green_val + nir_val)
                    } else {
                        -999.0 // NoData value
                    };
                });
                
                // Return result as TypedBuffer
                TypedBuffer::F32(gdal::raster::Buffer::new(shape, result_data))
            },
            // Add support for other types as needed
            _ => panic!("Unsupported input types for NDWI calculation"),
        }
    }
    
    fn required_bands(&self) -> usize {
        2 // NDWI requires exactly 2 bands
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}