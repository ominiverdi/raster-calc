// src/io/writer.rs
use anyhow::Result;
use gdal::{DriverManager, Metadata};
use gdal::raster::{Buffer, RasterCreationOptions};
use std::path::Path;

use crate::utils::fixed_point::to_fixed_point;
use super::reader::GeoInfo;

pub fn write_raster(
    chunks: Vec<((usize, usize), Vec<f32>)>,
    geo_info: GeoInfo,
    output_path: &Path,
    use_fixed_point: bool,
    scale_factor: i32,
) -> Result<()> {
    let driver = DriverManager::get_driver_by_name("GTiff")?;
    let chunk_size = chunks.first().map(|(_, data)| (data.len() as f64).sqrt() as usize).unwrap_or(0);
    
    let creation_options = RasterCreationOptions::from_iter([
        "COMPRESS=DEFLATE", 
        "TILED=YES", 
        "NUM_THREADS=ALL_CPUS"
    ]);
    
    if use_fixed_point {
        // Create int16 output
        let mut out_ds = driver.create_with_band_type_with_options::<i16, _>(
            output_path,
            geo_info.width,
            geo_info.height,
            1,
            &creation_options,
        )?;
        
        out_ds.set_projection(&geo_info.projection)?;
        out_ds.set_geo_transform(&geo_info.geo_transform)?;
        
        let mut band = out_ds.rasterband(1)?;
        let nodata_value: i16 = -10000;
        band.set_no_data_value(Some(nodata_value as f64))?;
        band.set_metadata_item("SCALE", &format!("{}", 1.0 / scale_factor as f64), "")?;
        band.set_metadata_item("OFFSET", "0", "")?;
        band.set_description("NDI (scaled)")?;
        
        // Write data chunks
        for (pos, data) in chunks {
            let (x, y) = pos;
            let x_pos = x * chunk_size;
            let y_pos = y * chunk_size;
            let width = (data.len() as f64).sqrt() as usize;
            let height = width;
            
            let fixed_data = to_fixed_point(&data, scale_factor, nodata_value);
            let mut buffer = Buffer::new((width, height), fixed_data);
            band.write((x_pos as isize, y_pos as isize), (width, height), &mut buffer)?;
        }
        
        out_ds.flush_cache()?;
    } else {
        // Create float32 output
        let mut out_ds = driver.create_with_band_type_with_options::<f32, _>(
            output_path,
            geo_info.width,
            geo_info.height,
            1,
            &creation_options,
        )?;
        
        out_ds.set_projection(&geo_info.projection)?;
        out_ds.set_geo_transform(&geo_info.geo_transform)?;
        
        let mut band = out_ds.rasterband(1)?;
        let nodata_value: f64 = -999.0;
        band.set_no_data_value(Some(nodata_value))?;
        band.set_description("NDI")?;
        
        // Write data chunks
        for (pos, data) in chunks {
            let (x, y) = pos;
            let x_pos = x * chunk_size;
            let y_pos = y * chunk_size;
            let width = (data.len() as f64).sqrt() as usize;
            let height = width;
            
            let mut buffer = Buffer::new((width, height), data);
            band.write((x_pos as isize, y_pos as isize), (width, height), &mut buffer)?;
        }
        
        out_ds.flush_cache()?;
    }
    
    Ok(())
}