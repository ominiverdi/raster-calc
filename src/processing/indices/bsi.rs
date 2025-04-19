// src/processing/indices/bsi.rs
use crate::utils::gdal_ext::TypedBuffer;
use crate::processing::parallel::IndexCalculator;
use rayon::prelude::*;

/// Bare Soil Index (BSI) calculator
/// BSI = ((SWIR + RED) - (NIR + BLUE)) / ((SWIR + RED) + (NIR + BLUE))
pub struct BSI {
    swir_index: usize,
    red_index: usize,
    nir_index: usize,
    blue_index: usize,
    name: String,
}

impl BSI {
    pub fn new(
        swir_index: usize, 
        red_index: usize, 
        nir_index: usize, 
        blue_index: usize, 
        name: Option<String>
    ) -> Self {
        Self {
            swir_index,
            red_index,
            nir_index,
            blue_index,
            name: name.unwrap_or_else(|| "BSI".to_string()),
        }
    }
}

impl IndexCalculator for BSI {
    fn calculate(&self, inputs: &[TypedBuffer]) -> TypedBuffer {
        // Extract input bands
        let swir = &inputs[self.swir_index];
        let red = &inputs[self.red_index];
        let nir = &inputs[self.nir_index];
        let blue = &inputs[self.blue_index];
        
        // Handle different input types (focusing on f32 for now)
        match (swir, red, nir, blue) {
            (TypedBuffer::F32(swir_data), TypedBuffer::F32(red_data), 
             TypedBuffer::F32(nir_data), TypedBuffer::F32(blue_data)) => {
                
                let shape = swir_data.shape();
                let swir_band = swir_data.data();
                let red_band = red_data.data();
                let nir_band = nir_data.data();
                let blue_band = blue_data.data();
                
                // Preallocate result buffer
                let mut result_data = vec![0.0f32; shape.0 * shape.1];
                
                // Calculate BSI in parallel
                result_data.par_iter_mut().enumerate().for_each(|(i, result)| {
                    let swir_val = swir_band[i];
                    let red_val = red_band[i];
                    let nir_val = nir_band[i];
                    let blue_val = blue_band[i];
                    
                    // Calculate BSI components
                    let numerator = (swir_val + red_val) - (nir_val + blue_val);
                    let denominator = (swir_val + red_val) + (nir_val + blue_val);
                    
                    // Handle division by zero or very small numbers
                    *result = if denominator.abs() > 1e-6 {
                        // Ensure result is within [-1, 1] range
                        (numerator / denominator).max(-1.0).min(1.0)
                    } else {
                        -999.0 // NoData value
                    };
                });
                
                // Return result as TypedBuffer
                TypedBuffer::F32(gdal::raster::Buffer::new(shape, result_data))
            },
            // Add support for other types as needed
            _ => panic!("Unsupported input types for BSI calculation"),
        }
    }
    
    fn required_bands(&self) -> usize {
        4 // BSI requires 4 bands (SWIR, RED, NIR, BLUE)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}