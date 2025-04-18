// src/io/reader.rs
use anyhow::Result;
use gdal::Dataset;
use gdal::raster::Buffer;
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

pub struct GeoInfo {
    pub projection: String,
    pub geo_transform: [f64; 6],
    pub width: usize,
    pub height: usize,
}

pub fn read_bands_parallel(band_paths: &[&Path]) -> Result<(Vec<((usize, usize), Vec<Buffer<f32>>)>, GeoInfo)> {
    // Open first dataset to get dimensions and projection info
    let first_ds = Dataset::open(band_paths[0])?;
    let (width, height) = first_ds.raster_size();
    let projection = first_ds.projection();
    let geo_transform = first_ds.geo_transform()?;
    
    // Get optimal chunk size based on cache line size and CPU count
    let _num_cpus = std::thread::available_parallelism()?.get();
    let chunk_size = 2048;  // Optimal based on benchmarks
    let chunks_x = (width as usize + chunk_size - 1) / chunk_size;
    let chunks_y = (height as usize + chunk_size - 1) / chunk_size;
    
    // Create shared datasets
    let datasets: Vec<Arc<Mutex<Dataset>>> = band_paths
        .iter()
        .map(|path| Arc::new(Mutex::new(Dataset::open(path).unwrap())))
        .collect();
    
    // Process chunks in parallel
    let chunks: Mutex<Vec<((usize, usize), Vec<Buffer<f32>>)>> = Mutex::new(Vec::new());
    
    (0..chunks_y).into_par_iter().for_each(|y| {
        (0..chunks_x).into_par_iter().for_each(|x| {
            let chunk_width = if (x + 1) * chunk_size > width as usize {
                width as usize - x * chunk_size
            } else {
                chunk_size
            };
            
            let chunk_height = if (y + 1) * chunk_size > height as usize {
                height as usize - y * chunk_size
            } else {
                chunk_size
            };
            
            let mut band_chunks = Vec::with_capacity(band_paths.len());
            
            for dataset in &datasets {
                let ds = dataset.lock().unwrap();
                let band = ds.rasterband(1).unwrap();
                
                let chunk_data = band.read_as::<f32>(
                    ((x * chunk_size) as isize, (y * chunk_size) as isize),
                    (chunk_width, chunk_height),
                    (chunk_width, chunk_height),
                    None,
                ).unwrap();
                
                band_chunks.push(chunk_data);
            }
            
            chunks.lock().unwrap().push(((x, y), band_chunks));
        });
    });
    
    let geo_info = GeoInfo {
        projection,
        geo_transform: geo_transform.try_into().unwrap(),
        width: width as usize,
        height: height as usize,
    };
    
    Ok((chunks.into_inner().unwrap(), geo_info))
}