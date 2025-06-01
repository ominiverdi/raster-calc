// src/processing/indices/msavi2.rs
use crate::utils::gdal_ext::TypedBuffer;
use crate::processing::parallel::IndexCalculator;
use rayon::prelude::*;

/// Modified Soil Adjusted Vegetation Index 2 (MSAVI2) calculator
/// MSAVI2 = (2 * NIR + 1 - sqrt((2 * NIR + 1)^2 - 8 * (NIR - RED))) / 2
pub struct MSAVI2 {
    nir_index: usize,
    red_index: usize,
    name: String,
}

impl MSAVI2 {
    pub fn new(nir_index: usize, red_index: usize, name: Option<String>) -> Self {
        Self {
            nir_index,
            red_index,
            name: name.unwrap_or_else(|| "MSAVI2".to_string()),
        }
    }
}

impl IndexCalculator for MSAVI2 {
    fn calculate(&self, inputs: &[TypedBuffer]) -> TypedBuffer {
        // Extract input bands
        let nir = &inputs[self.nir_index];
        let red = &inputs[self.red_index];
        
        // Handle different input types (focusing on f32 for now)
        match (nir, red) {
            (TypedBuffer::F32(nir_data), TypedBuffer::F32(red_data)) => {
                let shape = nir_data.shape();
                let nir_band = nir_data.data();
                let red_band = red_data.data();
                
                // Preallocate result buffer
                let mut result_data = vec![0.0f32; shape.0 * shape.1];
                
                // Calculate MSAVI2 in parallel
                // NOTE: Input scaling should be applied by the processor before calling this
                result_data.par_iter_mut().enumerate().for_each(|(i, result)| {
                    let nir_val = nir_band[i];
                    let red_val = red_band[i];
                    
                    // Check for valid data
                    if nir_val <= 0.0 && red_val <= 0.0 {
                        *result = -999.0; // NoData value
                    } else {
                        // Calculate MSAVI2
                        // (2 * NIR + 1 - sqrt((2 * NIR + 1)^2 - 8 * (NIR - RED))) / 2
                        let two_nir_plus_one = 2.0 * nir_val + 1.0;
                        let discriminant = (two_nir_plus_one * two_nir_plus_one) - 8.0 * (nir_val - red_val);
                        
                        if discriminant < 0.0 {
                            // Negative value under square root - invalid result
                            *result = -999.0;
                        } else {
                            *result = (two_nir_plus_one - discriminant.sqrt()) / 2.0;
                            
                            // Ensure result is in a reasonable range for vegetation indices
                            if *result < -1.0 || *result > 1.0 {
                                *result = -999.0;
                            }
                        }
                    }
                });
                
                // Return result as TypedBuffer
                TypedBuffer::F32(gdal::raster::Buffer::new(shape, result_data))
            },
            // Add support for other types as needed
            _ => panic!("Unsupported input types for MSAVI2 calculation"),
        }
    }
    
    fn required_bands(&self) -> usize {
        2 // MSAVI2 requires exactly 2 bands (NIR, RED)
    }
    
    fn name(&self) -> &str {
        &self.name
    }

    fn needs_input_scaling(&self) -> bool {
        true // MSAVI2 has constants (1, 2, 8) that require proper reflectance values
    }
}