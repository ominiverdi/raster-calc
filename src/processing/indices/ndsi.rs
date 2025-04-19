// src/processing/indices/ndsi.rs
use crate::utils::gdal_ext::TypedBuffer;
use crate::processing::parallel::IndexCalculator;
use rayon::prelude::*;

/// Normalized Difference Snow Index (NDSI) calculator
pub struct NDSI {
    green_index: usize,
    swir_index: usize,
    name: String,
}

impl NDSI {
    pub fn new(green_index: usize, swir_index: usize, name: Option<String>) -> Self {
        Self {
            green_index,
            swir_index,
            name: name.unwrap_or_else(|| "NDSI".to_string()),
        }
    }
}

impl IndexCalculator for NDSI {
    fn calculate(&self, inputs: &[TypedBuffer]) -> TypedBuffer {
        // Extract input bands
        let green = &inputs[self.green_index];
        let swir = &inputs[self.swir_index];
        
        // Handle different input types (focusing on f32 for now)
        match (green, swir) {
            (TypedBuffer::F32(green_data), TypedBuffer::F32(swir_data)) => {
                let shape = green_data.shape();
                let green_band = green_data.data();
                let swir_band = swir_data.data();
                
                // Preallocate result buffer
                let mut result_data = vec![0.0f32; shape.0 * shape.1];
                
                // Calculate NDSI in parallel (GREEN - SWIR) / (GREEN + SWIR)
                result_data.par_iter_mut().enumerate().for_each(|(i, result)| {
                    let green_val = green_band[i];
                    let swir_val = swir_band[i];
                    
                    // Handle division by zero or very small numbers
                    let sum = green_val + swir_val;
                    *result = if sum.abs() > 1e-6 {
                        // Ensure result is within [-1, 1] range
                        ((green_val - swir_val) / sum).max(-1.0).min(1.0)
                    } else {
                        -999.0 // NoData value
                    };
                });
                
                // Return result as TypedBuffer
                TypedBuffer::F32(gdal::raster::Buffer::new(shape, result_data))
            },
            // Add support for other types as needed
            _ => panic!("Unsupported input types for NDSI calculation"),
        }
    }
    
    fn required_bands(&self) -> usize {
        2 // NDSI requires exactly 2 bands
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}