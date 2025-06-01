// src/processing/indices/savi.rs
use crate::utils::gdal_ext::TypedBuffer;
use crate::processing::parallel::IndexCalculator;
use rayon::prelude::*;

/// Soil Adjusted Vegetation Index (SAVI) calculator
pub struct SAVI {
    nir_index: usize,
    red_index: usize,
    soil_factor: f32,
    name: String,
}

impl SAVI {
    pub fn new(nir_index: usize, red_index: usize, soil_factor: f32, name: Option<String>) -> Self {
        Self {
            nir_index,
            red_index,
            soil_factor,
            name: name.unwrap_or_else(|| "SAVI".to_string()),
        }
    }
}

impl IndexCalculator for SAVI {
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
                
                // Extract soil adjustment factor
                let l = self.soil_factor;
                
                // Calculate SAVI in parallel
                // NOTE: Input scaling should be applied by the processor before calling this
                result_data.par_iter_mut().enumerate().for_each(|(i, result)| {
                    let nir_val = nir_band[i];
                    let red_val = red_band[i];
                    
                    // Calculate SAVI: ((NIR - RED) / (NIR + RED + L)) * (1 + L)
                    let denominator = nir_val + red_val + l;
                    
                    *result = if denominator.abs() > 1e-3 {
                        let savi = ((nir_val - red_val) / denominator) * (1.0 + l);
                        savi.max(-1.0).min(1.0)  // Proper bounds
                    } else {
                        -999.0
                    };
                });
                
                // Return result as TypedBuffer
                TypedBuffer::F32(gdal::raster::Buffer::new(shape, result_data))
            },
            // Add support for other types as needed
            _ => panic!("Unsupported input types for SAVI calculation"),
        }
    }
    
    fn required_bands(&self) -> usize {
        2 // SAVI requires exactly 2 bands (NIR, RED)
    }
    
    fn name(&self) -> &str {
        &self.name
    }

    fn needs_input_scaling(&self) -> bool {
        true // SAVI has soil factor L (typically 0.5) that requires proper reflectance values
    }
}