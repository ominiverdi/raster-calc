// src/batch.rs
use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::processing::ParallelProcessor;
use crate::processing::indices::{NDI, EVI, SAVI, NDWI, NDSI, BSI, MSAVI2, OSAVI};

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
    pub params: OperationParams,
    pub output: String,
    pub float: Option<bool>,
    pub scale_factor: Option<i32>,
    pub compress: Option<String>,
    pub compress_level: Option<u8>,
    pub tiled: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum OperationParams {
    NdiParams { a: String, b: String },
    EviParams { a: String, b: String, c: String },
    SaviParams { a: String, b: String, l: Option<f32> },
    NdwiParams { a: String, b: String },
    NdsiParams { a: String, b: String },
    BsiParams { s: String, r: String, n: String, b: String },
    MsaviParams { a: String, b: String },
    OsaviParams { a: String, b: String },
}

pub fn process_batch(config_path: &PathBuf) -> Result<()> {
    // Read and parse configuration file
    let config_content = fs::read_to_string(config_path)?;
    let config: BatchConfig = serde_json::from_str(&config_content)?;
    
    // Create processor
    let processor = ParallelProcessor::new(None);
    
    println!("Starting batch processing with {} operations...", config.operations.len());
    
    // Process each operation
    for (i, op) in config.operations.iter().enumerate() {
        println!("[{}/{}] Processing {} -> {}", 
            i+1, config.operations.len(), op.op_type, op.output);
        
        // Get parameters, with operation-specific overrides
        let float = op.float.unwrap_or(config.global.float);
        let scale_factor = op.scale_factor.unwrap_or(config.global.scale_factor);
        let compress = op.compress.as_deref().unwrap_or(&config.global.compress);
        let compress_level = op.compress_level.unwrap_or(config.global.compress_level);
        let tiled = op.tiled.unwrap_or(config.global.tiled);
        
        match op.op_type.to_lowercase().as_str() {
            "ndi" => {
                if let OperationParams::NdiParams { a, b } = &op.params {
                    let ndi = NDI::new(0, 1, None);
                    processor.process(
                        ndi,
                        &[a.clone(), b.clone()],
                        &op.output,
                        !float,
                        scale_factor,
                        compress,
                        compress_level,
                        tiled,
                    )?;
                } else {
                    return Err(anyhow::anyhow!("Invalid parameters for NDI operation"));
                }
            },
            "evi" => {
                if let OperationParams::EviParams { a, b, c } = &op.params {
                    let evi = EVI::new(0, 1, 2, None);
                    processor.process(
                        evi,
                        &[a.clone(), b.clone(), c.clone()],
                        &op.output,
                        !float,
                        scale_factor,
                        compress,
                        compress_level,
                        tiled,
                    )?;
                } else {
                    return Err(anyhow::anyhow!("Invalid parameters for EVI operation"));
                }
            },
            "savi" => {
                if let OperationParams::SaviParams { a, b, l } = &op.params {
                    let soil_factor = l.unwrap_or(0.5);
                    let savi = SAVI::new(0, 1, soil_factor, None);
                    processor.process(
                        savi,
                        &[a.clone(), b.clone()],
                        &op.output,
                        !float,
                        scale_factor,
                        compress,
                        compress_level,
                        tiled,
                    )?;
                } else {
                    return Err(anyhow::anyhow!("Invalid parameters for SAVI operation"));
                }
            },
            "ndwi" => {
                if let OperationParams::NdwiParams { a, b } = &op.params {
                    let ndwi = NDWI::new(0, 1, None);
                    processor.process(
                        ndwi,
                        &[a.clone(), b.clone()],
                        &op.output,
                        !float,
                        scale_factor,
                        compress,
                        compress_level,
                        tiled,
                    )?;
                } else {
                    return Err(anyhow::anyhow!("Invalid parameters for NDWI operation"));
                }
            },
            "ndsi" => {
                if let OperationParams::NdsiParams { a, b } = &op.params {
                    let ndsi = NDSI::new(0, 1, None);
                    processor.process(
                        ndsi,
                        &[a.clone(), b.clone()],
                        &op.output,
                        !float,
                        scale_factor,
                        compress,
                        compress_level,
                        tiled,
                    )?;
                } else {
                    return Err(anyhow::anyhow!("Invalid parameters for NDSI operation"));
                }
            },
            "bsi" => {
                if let OperationParams::BsiParams { s, r, n, b } = &op.params {
                    let bsi = BSI::new(0, 1, 2, 3, None);
                    processor.process(
                        bsi,
                        &[s.clone(), r.clone(), n.clone(), b.clone()],
                        &op.output,
                        !float,
                        scale_factor,
                        compress,
                        compress_level,
                        tiled,
                    )?;
                } else {
                    return Err(anyhow::anyhow!("Invalid parameters for BSI operation"));
                }
            },
            "msavi2" => {
                if let OperationParams::MsaviParams { a, b } = &op.params {
                    let msavi2 = MSAVI2::new(0, 1, None);
                    processor.process(
                        msavi2,
                        &[a.clone(), b.clone()],
                        &op.output,
                        !float,
                        scale_factor,
                        compress,
                        compress_level,
                        tiled,
                    )?;
                } else {
                    return Err(anyhow::anyhow!("Invalid parameters for MSAVI2 operation"));
                }
            },
            "osavi" => {
                if let OperationParams::OsaviParams { a, b } = &op.params {
                    let osavi = OSAVI::new(0, 1, None);
                    processor.process(
                        osavi,
                        &[a.clone(), b.clone()],
                        &op.output,
                        !float,
                        scale_factor,
                        compress,
                        compress_level,
                        tiled,
                    )?;
                } else {
                    return Err(anyhow::anyhow!("Invalid parameters for OSAVI operation"));
                }
            },
            _ => return Err(anyhow::anyhow!("Unknown operation type: {}", op.op_type)),
        }
    }
    
    println!("Batch processing complete!");
    Ok(())
}