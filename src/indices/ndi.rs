use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

use crate::io::{read_bands_parallel, write_raster};

pub fn calculate_ndi(
    band_a_path: &Path,
    band_b_path: &Path,
    output_path: &Path,
    use_fixed_point: bool,
    scale_factor: i32,
) -> Result<()> {
    // Read bands in parallel chunks
    let (chunks, geo_info) = read_bands_parallel(&[band_a_path, band_b_path])?;
    
    // Process chunks in parallel
    let result_chunks = chunks.into_par_iter().map(|(pos, blocks)| {
        let band_a = &blocks[0];
        let band_b = &blocks[1];
        let shape = band_a.shape();
        let mut result = vec![0.0f32; shape.0 * shape.1];
        
        // Calculate NDI for each pixel
        for i in 0..result.len() {
            let a = band_a.data()[i];
            let b = band_b.data()[i];
            
            result[i] = if a + b != 0.0 {
                (a - b) / (a + b)
            } else {
                -999.0 // NoData
            };
        }
        
        (pos, result)
    }).collect();
    
    // Write result to file
    write_raster(
        result_chunks,
        geo_info,
        output_path,
        use_fixed_point,
        scale_factor,
    )
}