// src/processing/indices/osavi.rs
use crate::utils::gdal_ext::TypedBuffer;
use crate::processing::parallel::IndexCalculator;
use rayon::prelude::*;

/// Optimized Soil Adjusted Vegetation Index (OSAVI) calculator
/// OSAVI = (NIR - RED) / (NIR + RED + 0.16) * (1 + 0.16)
pub struct OSAVI {
    nir_index: usize,
    red_index: usize,
    name: String,
}

impl OSAVI {
    pub fn new(nir_index: usize, red_index: usize, name: Option<String>) -> Self {
        Self {
            nir_index,
            red_index,
            name: name.unwrap_or_else(|| "OSAVI".to_string()),
        }
    }
}

impl IndexCalculator for OSAVI {
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
                
                // Fixed soil adjustment factor for OSAVI
                const L: f32 = 0.16;
                
                // Calculate OSAVI in parallel
                // NOTE: Input scaling should be applied by the processor before calling this
                result_data.par_iter_mut().enumerate().for_each(|(i, result)| {
                    let nir_val = nir_band[i];
                    let red_val = red_band[i];
                    
                    // Calculate OSAVI: ((NIR - RED) / (NIR + RED + L)) * (1 + L)
                    let denominator = nir_val + red_val + L;
                    
                    *result = if denominator.abs() > 1e-6 {
                        ((nir_val - red_val) / denominator) * (1.0 + L)
                    } else {
                        -999.0 // NoData value
                    };
                });
                
                // Return result as TypedBuffer
                TypedBuffer::F32(gdal::raster::Buffer::new(shape, result_data))
            },
            // Add support for other types as needed
            _ => panic!("Unsupported input types for OSAVI calculation"),
        }
    }
    
    fn required_bands(&self) -> usize {
        2 // OSAVI requires exactly 2 bands (NIR, RED)
    }
    
    fn name(&self) -> &str {
        &self.name
    }

    fn needs_input_scaling(&self) -> bool {
        true // OSAVI has constant L=0.16 that requires proper reflectance values
    }
}