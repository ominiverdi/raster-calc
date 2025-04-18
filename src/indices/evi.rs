// src/indices/evi.rs
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

use crate::io::{read_bands_parallel, write_raster};

pub fn calculate_evi(
    nir_path: &Path,
    red_path: &Path,
    blue_path: &Path,
    output_path: &Path,
    use_fixed_point: bool,
    scale_factor: i32,
) -> Result<()> {
    // EVI constants
    const G: f32 = 2.5;  // Gain factor
    const C1: f32 = 6.0; // Coefficient 1
    const C2: f32 = 7.5; // Coefficient 2
    const L: f32 = 1.0;  // Canopy background adjustment
    
    // Read bands in parallel chunks
    let (chunks, geo_info) = read_bands_parallel(&[nir_path, red_path, blue_path])?;
    
    // Process chunks in parallel
    let result_chunks = chunks.into_par_iter().map(|(pos, blocks)| {
        let nir = &blocks[0];
        let red = &blocks[1];
        let blue = &blocks[2];
        let shape = nir.shape();
        let mut result = vec![0.0f32; shape.0 * shape.1];
        
        // Calculate EVI for each pixel
        for i in 0..result.len() {
            let nir_val = nir.data()[i];
            let red_val = red.data()[i];
            let blue_val = blue.data()[i];
            
            let denominator = nir_val + C1 * red_val - C2 * blue_val + L;
            
            result[i] = if denominator != 0.0 {
                G * (nir_val - red_val) / denominator
            } else {
                -999.0 // NoData
            };
            
            // Clamp values to valid EVI range
            if result[i] != -999.0 {
                result[i] = result[i].max(-1.0).min(1.0);
            }
        }
        
        (pos, result)
    }).collect();
    
    write_raster(result_chunks, geo_info, output_path, use_fixed_point, scale_factor)
}