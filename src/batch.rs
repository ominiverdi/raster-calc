// src/batch.rs
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::utils::cache::RasterCache;
use std::collections::HashSet;

use crate::processing::indices::{BSI, EVI, MSAVI2, NDI, NDSI, NDWI, OSAVI, SAVI};
use crate::processing::ParallelProcessor;

#[derive(Deserialize, Serialize, Debug)]
pub struct BatchConfig {
    #[serde(default)]
    pub global: GlobalParams,
    pub operations: Vec<Operation>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct GlobalParams {
    #[serde(default = "default_compress")]
    pub compress: String,
    #[serde(default = "default_compress_level")]
    pub compress_level: u8,
    #[serde(default)]
    pub float: bool,
    #[serde(default = "default_scale_factor")]
    pub scale_factor: i32,
    #[serde(default = "default_true")]
    pub tiled: bool,
}

fn default_compress() -> String {
    "DEFLATE".to_string()
}

fn default_compress_level() -> u8 {
    6
}

fn default_scale_factor() -> i32 {
    10000
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Operation {
    #[serde(rename = "type")]
    pub op_type: String,
    pub params: Value,
    pub output: String,
    pub float: Option<bool>,
    pub scale_factor: Option<i32>,
    pub compress: Option<String>,
    pub compress_level: Option<u8>,
    pub tiled: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct EviParams {
    pub a: String,
    pub b: String,
    pub c: String,
}

#[derive(Deserialize, Debug)]
pub struct NdiParams {
    pub a: String,
    pub b: String,
}

#[derive(Deserialize, Debug)]
pub struct SaviParams {
    pub a: String,
    pub b: String,
    pub l: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct NdwiParams {
    pub a: String,
    pub b: String,
}

#[derive(Deserialize, Debug)]
pub struct NdsiParams {
    pub a: String,
    pub b: String,
}

#[derive(Deserialize, Debug)]
pub struct BsiParams {
    pub s: String,
    pub r: String,
    pub n: String,
    pub b: String,
}

#[derive(Deserialize, Debug)]
pub struct MsaviParams {
    pub a: String,
    pub b: String,
}

#[derive(Deserialize, Debug)]
pub struct OsaviParams {
    pub a: String,
    pub b: String,
}

pub fn process_batch(config_path: &PathBuf) -> Result<()> {
    let config_content = fs::read_to_string(config_path)?;
    let config: BatchConfig = serde_json::from_str(&config_content)?;

    // Create shared cache
    let cache = Arc::new(RasterCache::new());
    
    // Collect all unique paths
    let unique_paths: Vec<String> = collect_unique_paths(&config).into_iter().collect();
    println!("Found {} unique input files", unique_paths.len());
    
    // Prefetch datasets (optional but helpful)
    for path in &unique_paths {
        if let Err(e) = cache.get_dataset(path) {
            eprintln!("Warning: Could not preload {}: {}", path, e);
        }
    }
    println!("Cache initialized with {} datasets", cache.len());
    
    // Create processor with cache
    let processor = ParallelProcessor::with_cache(None, Arc::clone(&cache));

    println!("Starting batch processing with {} operations...", config.operations.len());

    for (i, op) in config.operations.iter().enumerate() {
        println!("[{}/{}] Processing {} -> {}", i + 1, config.operations.len(), op.op_type, op.output);

        let float = op.float.unwrap_or(config.global.float);
        let scale_factor = op.scale_factor.unwrap_or(config.global.scale_factor);
        let compress = op.compress.as_deref().unwrap_or(&config.global.compress);
        let compress_level = op.compress_level.unwrap_or(config.global.compress_level);
        let tiled = op.tiled.unwrap_or(config.global.tiled);

        let result = match op.op_type.to_lowercase().as_str() {
            "ndi" => {
                let p: NdiParams = serde_json::from_value(op.params.clone())
                    .context("Invalid parameters for NDI operation")?;
                let alg = NDI::new(0, 1, None);
                processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, compress, compress_level, tiled)
            },
            "evi" => {
                let p: EviParams = serde_json::from_value(op.params.clone())
                    .context("Invalid parameters for EVI operation")?;
                let alg = EVI::new(0, 1, 2, None);
                processor.process(alg, &[p.a, p.b, p.c], &op.output, !float, scale_factor, compress, compress_level, tiled)
            },
            "savi" => {
                let p: SaviParams = serde_json::from_value(op.params.clone())
                    .context("Invalid parameters for SAVI operation")?;
                let alg = SAVI::new(0, 1, p.l.unwrap_or(0.5), None);
                processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, compress, compress_level, tiled)
            },
            "ndwi" => {
                let p: NdwiParams = serde_json::from_value(op.params.clone())
                    .context("Invalid parameters for NDWI operation")?;
                let alg = NDWI::new(0, 1, None);
                processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, compress, compress_level, tiled)
            },
            "ndsi" => {
                let p: NdsiParams = serde_json::from_value(op.params.clone())
                    .context("Invalid parameters for NDSI operation")?;
                let alg = NDSI::new(0, 1, None);
                processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, compress, compress_level, tiled)
            },
            "bsi" => {
                let p: BsiParams = serde_json::from_value(op.params.clone())
                    .context("Invalid parameters for BSI operation")?;
                let alg = BSI::new(0, 1, 2, 3, None);
                processor.process(alg, &[p.s, p.r, p.n, p.b], &op.output, !float, scale_factor, compress, compress_level, tiled)
            },
            "msavi2" => {
                let p: MsaviParams = serde_json::from_value(op.params.clone())
                    .context("Invalid parameters for MSAVI2 operation")?;
                let alg = MSAVI2::new(0, 1, None);
                processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, compress, compress_level, tiled)
            },
            "osavi" => {
                let p: OsaviParams = serde_json::from_value(op.params.clone())
                    .context("Invalid parameters for OSAVI operation")?;
                let alg = OSAVI::new(0, 1, None);
                processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, compress, compress_level, tiled)
            },
            _ => Err(anyhow::anyhow!("Unknown operation type: {}", op.op_type)),
        };

        if let Err(e) = result {
            return Err(anyhow::anyhow!("Error processing operation {}: {}", i + 1, e));
        }
    }

    println!("Batch processing complete with {} cached datasets", processor.cache_size());
    processor.clear_cache();
    Ok(())
}

fn collect_unique_paths(config: &BatchConfig) -> HashSet<String> {
    let mut paths = HashSet::new();

    for op in &config.operations {
        match op.op_type.to_lowercase().as_str() {
            "ndi" => {
                if let Ok(p) = serde_json::from_value::<NdiParams>(op.params.clone()) {
                    paths.insert(p.a);
                    paths.insert(p.b);
                }
            }
            "evi" => {
                if let Ok(p) = serde_json::from_value::<EviParams>(op.params.clone()) {
                    paths.insert(p.a);
                    paths.insert(p.b);
                    paths.insert(p.c);
                }
            }
            "savi" => {
                if let Ok(p) = serde_json::from_value::<SaviParams>(op.params.clone()) {
                    paths.insert(p.a);
                    paths.insert(p.b);
                }
            }
            // Add other index types
            "ndwi" => {
                if let Ok(p) = serde_json::from_value::<NdwiParams>(op.params.clone()) {
                    paths.insert(p.a);
                    paths.insert(p.b);
                }
            }
            "ndsi" => {
                if let Ok(p) = serde_json::from_value::<NdsiParams>(op.params.clone()) {
                    paths.insert(p.a);
                    paths.insert(p.b);
                }
            }
            "bsi" => {
                if let Ok(p) = serde_json::from_value::<BsiParams>(op.params.clone()) {
                    paths.insert(p.s);
                    paths.insert(p.r);
                    paths.insert(p.n);
                    paths.insert(p.b);
                }
            }
            "msavi2" => {
                if let Ok(p) = serde_json::from_value::<MsaviParams>(op.params.clone()) {
                    paths.insert(p.a);
                    paths.insert(p.b);
                }
            }
            "osavi" => {
                if let Ok(p) = serde_json::from_value::<OsaviParams>(op.params.clone()) {
                    paths.insert(p.a);
                    paths.insert(p.b);
                }
            }
            _ => {}
        }
    }

    paths
}
