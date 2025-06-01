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
        
        match (nir, red, blue) {
            (TypedBuffer::F32(nir_data), TypedBuffer::F32(red_data), TypedBuffer::F32(blue_data)) => {
                let shape = nir_data.shape();
                let nir_band = nir_data.data();
                let red_band = red_data.data();
                let blue_band = blue_data.data();
                
                let mut result_data = vec![0.0f32; shape.0 * shape.1];
                
                // EVI coefficients from MODIS documentation
                const G: f32 = 2.5;    // Gain factor
                const L: f32 = 1.0;    // Soil adjustment factor
                const C1: f32 = 6.0;   // Aerosol resistance (red)
                const C2: f32 = 7.5;   // Aerosol resistance (blue)
                
                result_data.par_iter_mut().enumerate().for_each(|(i, result)| {
                    let nir_val = nir_band[i];
                    let red_val = red_band[i];
                    let blue_val = blue_band[i];
                    
                    // Basic sanity check - reject clearly invalid values
                    if nir_val < -1000.0 || red_val < -1000.0 || blue_val < -1000.0 ||
                       nir_val > 50000.0 || red_val > 50000.0 || blue_val > 50000.0 {
                        *result = -999.0;
                        return;
                    }
                    
                    // Handle negative values (atmospheric correction artifacts)
                    let nir_clean = nir_val.max(0.0);
                    let red_clean = red_val.max(0.0);
                    let blue_clean = blue_val.max(0.0);
                    
                    // Check for blue band saturation in the actual data range
                    // For DN values, saturation threshold is much higher
                    let blue_saturation_threshold = if blue_clean > 10.0 { 2000.0 } else { 0.25 };
                    
                    if blue_clean >= blue_saturation_threshold {
                        // Use 2-band EVI backup formula
                        let denominator_2band = nir_clean + 2.4 * red_clean + 1.0;
                        *result = if denominator_2band > 1e-3 {
                            let evi2 = G * (nir_clean - red_clean) / denominator_2band;
                            evi2.max(-0.2).min(1.0)
                        } else {
                            -999.0
                        };
                    } else {
                        // Use standard 3-band EVI
                        let denominator = nir_clean + C1 * red_clean - C2 * blue_clean + L;
                        *result = if denominator > 1e-3 {
                            let evi = G * (nir_clean - red_clean) / denominator;
                            // Clamp to valid EVI range [-0.2, 1.0]
                            evi.max(-0.2).min(1.0)
                        } else {
                            -999.0
                        };
                    }
                });
                
                TypedBuffer::F32(gdal::raster::Buffer::new(shape, result_data))
            },
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