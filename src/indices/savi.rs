// src/indices/savi.rs
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

use crate::io::{read_bands_parallel, write_raster};

pub fn calculate_savi(
    nir_path: &Path,
    red_path: &Path,
    output_path: &Path,
    soil_factor: f32,
    use_fixed_point: bool,
    scale_factor: i32,
) -> Result<()> {
    // Read bands in parallel chunks
    let (chunks, geo_info) = read_bands_parallel(&[nir_path, red_path])?;
    
    // Process chunks in parallel
    let result_chunks = chunks.into_par_iter().map(|(pos, blocks)| {
        let nir = &blocks[0];
        let red = &blocks[1];
        let shape = nir.shape();
        let mut result = vec![0.0f32; shape.0 * shape.1];
        
        // Calculate SAVI for each pixel
        for i in 0..result.len() {
            let nir_val = nir.data()[i];
            let red_val = red.data()[i];
            
            let denominator = nir_val + red_val + soil_factor;
            
            result[i] = if denominator != 0.0 {
                // SAVI = ((NIR - RED) / (NIR + RED + L)) * (1 + L)
                ((nir_val - red_val) / denominator) * (1.0 + soil_factor)
            } else {
                -999.0 // NoData
            };
        }
        
        (pos, result)
    }).collect();
    
    write_raster(result_chunks, geo_info, output_path, use_fixed_point, scale_factor)
}