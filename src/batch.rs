// src/batch.rs
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;


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
    #[serde(default)]
    pub threads: Option<usize>,
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



    // Get thread count from config or use default calculation
    let thread_count = config.global.threads.unwrap_or_else(|| {
        std::cmp::max(4, (num_cpus::get() as f32 * 0.6) as usize)
    });
    
    println!("Configuring thread pool with {} threads (of {} available)", 
             thread_count, num_cpus::get());
             
    // Configure thread pool
    ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build_global()
        .unwrap_or_else(|e| eprintln!("Warning: Thread pool configuration failed: {}", e));
    
    // Create shared cache
    let cache = Arc::new(RasterCache::new());
    
    // Collect all unique paths
    let unique_paths: Vec<String> = collect_unique_paths(&config).into_iter().collect();
    println!("Found {} unique input files", unique_paths.len());
    
    // Prefetch datasets
    for path in &unique_paths {
        if let Err(e) = cache.get_dataset(path) {
            eprintln!("Warning: Could not preload {}: {}", path, e);
        }
    }
    println!("Cache initialized with {} datasets", cache.len());
    
    println!("Starting parallel batch processing with {} operations...", config.operations.len());
    
    // Track errors across parallel operations
    let errors = Arc::new(Mutex::new(Vec::new()));
    
    // Process operations in parallel using rayon
    config.operations.par_iter().enumerate().for_each(|(i, op)| {
        println!("[{}/{}] Processing {} -> {}", i + 1, config.operations.len(), op.op_type, op.output);
        
        // Create a processor for each parallel operation with the shared cache
        let processor = ParallelProcessor::with_cache(None, Arc::clone(&cache));
        
        // Get operation parameters
        let float = op.float.unwrap_or(config.global.float);
        let scale_factor = op.scale_factor.unwrap_or(config.global.scale_factor);
        let compress = op.compress.as_deref().unwrap_or(&config.global.compress);
        let compress_level = op.compress_level.unwrap_or(config.global.compress_level);
        let tiled = op.tiled.unwrap_or(config.global.tiled);
        
        // Process based on operation type
        let result = match op.op_type.to_lowercase().as_str() {
            "ndi" => {
                match serde_json::from_value::<NdiParams>(op.params.clone()) {
                    Ok(p) => {
                        let alg = NDI::new(0, 1, None);
                        processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, 
                                       compress, compress_level, tiled)
                    },
                    Err(e) => {
                        let mut error_list = errors.lock().unwrap();
                        error_list.push(format!("Error parsing NDI params for operation {}: {}", i + 1, e));
                        return;
                    }
                }
            },
            "evi" => {
                match serde_json::from_value::<EviParams>(op.params.clone()) {
                    Ok(p) => {
                        let alg = EVI::new(0, 1, 2, None);
                        processor.process(alg, &[p.a, p.b, p.c], &op.output, !float, scale_factor, 
                                       compress, compress_level, tiled)
                    },
                    Err(e) => {
                        let mut error_list = errors.lock().unwrap();
                        error_list.push(format!("Error parsing EVI params for operation {}: {}", i + 1, e));
                        return;
                    }
                }
            },
            "savi" => {
                match serde_json::from_value::<SaviParams>(op.params.clone()) {
                    Ok(p) => {
                        let alg = SAVI::new(0, 1, p.l.unwrap_or(0.5), None);
                        processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, 
                                       compress, compress_level, tiled)
                    },
                    Err(e) => {
                        let mut error_list = errors.lock().unwrap();
                        error_list.push(format!("Error parsing SAVI params for operation {}: {}", i + 1, e));
                        return;
                    }
                }
            },
            "ndwi" => {
                match serde_json::from_value::<NdwiParams>(op.params.clone()) {
                    Ok(p) => {
                        let alg = NDWI::new(0, 1, None);
                        processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, 
                                       compress, compress_level, tiled)
                    },
                    Err(e) => {
                        let mut error_list = errors.lock().unwrap();
                        error_list.push(format!("Error parsing NDWI params for operation {}: {}", i + 1, e));
                        return;
                    }
                }
            },
            "ndsi" => {
                match serde_json::from_value::<NdsiParams>(op.params.clone()) {
                    Ok(p) => {
                        let alg = NDSI::new(0, 1, None);
                        processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, 
                                       compress, compress_level, tiled)
                    },
                    Err(e) => {
                        let mut error_list = errors.lock().unwrap();
                        error_list.push(format!("Error parsing NDSI params for operation {}: {}", i + 1, e));
                        return;
                    }
                }
            },
            "bsi" => {
                match serde_json::from_value::<BsiParams>(op.params.clone()) {
                    Ok(p) => {
                        let alg = BSI::new(0, 1, 2, 3, None);
                        processor.process(alg, &[p.s, p.r, p.n, p.b], &op.output, !float, scale_factor, 
                                       compress, compress_level, tiled)
                    },
                    Err(e) => {
                        let mut error_list = errors.lock().unwrap();
                        error_list.push(format!("Error parsing BSI params for operation {}: {}", i + 1, e));
                        return;
                    }
                }
            },
            "msavi2" => {
                match serde_json::from_value::<MsaviParams>(op.params.clone()) {
                    Ok(p) => {
                        let alg = MSAVI2::new(0, 1, None);
                        processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, 
                                       compress, compress_level, tiled)
                    },
                    Err(e) => {
                        let mut error_list = errors.lock().unwrap();
                        error_list.push(format!("Error parsing MSAVI2 params for operation {}: {}", i + 1, e));
                        return;
                    }
                }
            },
            "osavi" => {
                match serde_json::from_value::<OsaviParams>(op.params.clone()) {
                    Ok(p) => {
                        let alg = OSAVI::new(0, 1, None);
                        processor.process(alg, &[p.a, p.b], &op.output, !float, scale_factor, 
                                       compress, compress_level, tiled)
                    },
                    Err(e) => {
                        let mut error_list = errors.lock().unwrap();
                        error_list.push(format!("Error parsing OSAVI params for operation {}: {}", i + 1, e));
                        return;
                    }
                }
            },
            _ => {
                let mut error_list = errors.lock().unwrap();
                error_list.push(format!("Unknown operation type for operation {}: {}", i + 1, op.op_type));
                return;
            }
        };
        
        if let Err(e) = result {
            let mut error_list = errors.lock().unwrap();
            error_list.push(format!("Error processing operation {}: {}", i + 1, e));
        }
    });
    
    // Check if any errors occurred
    let error_list = errors.lock().unwrap();
    if !error_list.is_empty() {
        for error in error_list.iter() {
            eprintln!("{}", error);
        }
        return Err(anyhow::anyhow!("Errors occurred during batch processing"));
    }
    
    println!("Batch processing complete with {} cached datasets", cache.len());
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
