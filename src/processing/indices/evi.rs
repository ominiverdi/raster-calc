// src/processing/indices/evi.rs
use crate::utils::gdal_ext::TypedBuffer;
use crate::processing::parallel::IndexCalculator;
use rayon::prelude::*;

/// Enhanced Vegetation Index (EVI) calculator
pub struct EVI {
    nir_index: usize,
    red_index: usize,
    blue_index: usize,
    name: String,
}

impl EVI {
    pub fn new(nir_index: usize, red_index: usize, blue_index: usize, name: Option<String>) -> Self {
        Self {
            nir_index,
            red_index,
            blue_index,
            name: name.unwrap_or_else(|| "EVI".to_string()),
        }
    }
}

impl IndexCalculator for EVI {
    fn calculate(&self, inputs: &[TypedBuffer]) -> TypedBuffer {
        // Extract input bands
        let nir = &inputs[self.nir_index];
        let red = &inputs[self.red_index];
        let blue = &inputs[self.blue_index];
        
        // Handle different input types (focusing on f32 for now)
        match (nir, red, blue) {
            (TypedBuffer::F32(nir_data), TypedBuffer::F32(red_data), TypedBuffer::F32(blue_data)) => {
                let shape = nir_data.shape();
                let nir_band = nir_data.data();
                let red_band = red_data.data();
                let blue_band = blue_data.data();
                
                // Preallocate result buffer
                let mut result_data = vec![0.0f32; shape.0 * shape.1];
                
                // Constants for EVI calculation
                const G: f32 = 2.5;    // Gain factor
                const L: f32 = 1.0;    // Soil adjustment factor
                const C1: f32 = 6.0;   // Coefficient for the aerosol resistance (red band)
                const C2: f32 = 7.5;   // Coefficient for the aerosol resistance (blue band)
                
                // Calculate EVI in parallel
                // NOTE: Input scaling should be applied by the processor before calling this
                result_data.par_iter_mut().enumerate().for_each(|(i, result)| {
                    let nir_val = nir_band[i];
                    let red_val = red_band[i];
                    let blue_val = blue_band[i];
                    
                    // Check for valid values
                    *result = if nir_val > 0.0 || red_val > 0.0 || blue_val > 0.0 {
                        let denominator = nir_val + C1 * red_val - C2 * blue_val + L;
                        
                        // Calculate EVI, handling possible division by zero
                        if denominator.abs() > 1e-6 {
                            G * (nir_val - red_val) / denominator
                        } else {
                            -999.0 // NoData value
                        }
                    } else {
                        -999.0 // NoData value
                    };
                });
                
                // Return result as TypedBuffer
                TypedBuffer::F32(gdal::raster::Buffer::new(shape, result_data))
            },
            // Add support for other types as needed
            _ => panic!("Unsupported input types for EVI calculation"),
        }
    }
    
    fn required_bands(&self) -> usize {
        3 // EVI requires exactly 3 bands (NIR, RED, BLUE)
    }
    
    fn name(&self) -> &str {
        &self.name
    }

    fn needs_input_scaling(&self) -> bool {
        true // EVI has constants (L=1.0, C1=6.0, C2=7.5) that require proper reflectance values
    }
}