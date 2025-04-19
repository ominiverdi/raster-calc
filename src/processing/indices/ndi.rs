// src/processing/indices/ndi.rs
use crate::utils::gdal_ext::TypedBuffer;
use crate::processing::parallel::IndexCalculator;
use rayon::prelude::*;

/// Normalized Difference Index (NDI) calculator
pub struct NDI {
    band_a_index: usize,
    band_b_index: usize,
    name: String,
}

impl NDI {
    pub fn new(band_a_index: usize, band_b_index: usize, name: Option<String>) -> Self {
        Self {
            band_a_index,
            band_b_index,
            name: name.unwrap_or_else(|| "NDI".to_string()),
        }
    }
}

impl IndexCalculator for NDI {
    fn calculate(&self, inputs: &[TypedBuffer]) -> TypedBuffer {
        // Extract input bands
        let band_a = &inputs[self.band_a_index];
        let band_b = &inputs[self.band_b_index];
        
        // Handle different input types (focusing on f32 for now)
        match (band_a, band_b) {
            (TypedBuffer::F32(a), TypedBuffer::F32(b)) => {
                let shape = a.shape();
                let a_data = a.data();
                let b_data = b.data();
                
                // Preallocate result buffer
                let mut result_data = vec![0.0f32; shape.0 * shape.1];
                
                // Calculate NDI in parallel
                result_data.par_iter_mut().enumerate().for_each(|(i, result)| {
                    let a_val = a_data[i];
                    let b_val = b_data[i];
                    
                    *result = if a_val + b_val > 0.0 {
                        (a_val - b_val) / (a_val + b_val)
                    } else {
                        -999.0 // NoData value
                    };
                });
                
                // Return result as TypedBuffer
                TypedBuffer::F32(gdal::raster::Buffer::new(shape, result_data))
            },
            // Add support for other types as needed
            _ => panic!("Unsupported input types for NDI calculation"),
        }
    }
    
    fn required_bands(&self) -> usize {
        2 // NDI requires exactly 2 bands
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}