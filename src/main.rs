// src/main.rs
use anyhow::Result;
use clap::Parser;

mod cli;
mod utils;
mod processing;
mod batch;

use crate::cli::{Cli, Commands};
use crate::processing::{ParallelProcessor, indices::{NDI, EVI, SAVI, NDWI, NDSI, BSI, MSAVI2, OSAVI}};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let processor = ParallelProcessor::new(None);

    match &cli.command {
        Commands::Batch { config } => {
            batch::process_batch(config)?;
        }
        
        Commands::Ndi { band_a, band_b } => {
            // Create NDI calculator with band_a as first and band_b as second
            let ndi = NDI::new(0, 1, None);
            
            processor.process(
                ndi,
                &[band_a.to_string_lossy().to_string(), band_b.to_string_lossy().to_string()],
                cli.output.to_string_lossy().as_ref(),
                !cli.float,
                cli.scale_factor,
                cli.input_scale_factor,
                &cli.compress,          
                cli.compress_level,     
                cli.tiled,              
            )?;
        },
        Commands::Evi { nir, red, blue } => {
            // Create EVI calculator with NIR, Red, and Blue bands
            let evi = EVI::new(0, 1, 2, None);
            
            processor.process(
                evi,
                &[
                    nir.to_string_lossy().to_string(),
                    red.to_string_lossy().to_string(),
                    blue.to_string_lossy().to_string()
                ],
                cli.output.to_string_lossy().as_ref(),
                !cli.float,
                cli.scale_factor,
                cli.input_scale_factor,
                &cli.compress,          
                cli.compress_level,     
                cli.tiled,          
            )?;
        },
        Commands::Savi { nir, red, soil_factor } => {
            // Create SAVI calculator with NIR and Red bands plus soil factor
            let savi = SAVI::new(0, 1, *soil_factor, None);
            
            processor.process(
                savi,
                &[
                    nir.to_string_lossy().to_string(),
                    red.to_string_lossy().to_string()
                ],
                cli.output.to_string_lossy().as_ref(),
                !cli.float,
                cli.scale_factor,
                cli.input_scale_factor,
                &cli.compress,          
                cli.compress_level,     
                cli.tiled,          
            )?;
        },
        Commands::Ndwi { green, nir } => {
            // Create NDWI calculator with Green and NIR bands
            let ndwi = NDWI::new(0, 1, None);
            
            processor.process(
                ndwi,
                &[
                    green.to_string_lossy().to_string(),
                    nir.to_string_lossy().to_string()
                ],
                cli.output.to_string_lossy().as_ref(),
                !cli.float,
                cli.scale_factor,
                cli.input_scale_factor,
                &cli.compress,          
                cli.compress_level,     
                cli.tiled,          
            )?;
        },
        Commands::Ndsi { green, swir } => {
            // Create NDSI calculator with Green and SWIR bands
            let ndsi = NDSI::new(0, 1, None);
            
            processor.process(
                ndsi,
                &[
                    green.to_string_lossy().to_string(),
                    swir.to_string_lossy().to_string()
                ],
                cli.output.to_string_lossy().as_ref(),
                !cli.float,
                cli.scale_factor,
                cli.input_scale_factor,
                &cli.compress,          
                cli.compress_level,     
                cli.tiled,          
            )?;
        },
        Commands::Bsi { swir, red, nir, blue } => {
            // Create BSI calculator with SWIR, RED, NIR, and BLUE bands
            let bsi = BSI::new(0, 1, 2, 3, None);
            
            processor.process(
                bsi,
                &[
                    swir.to_string_lossy().to_string(),
                    red.to_string_lossy().to_string(),
                    nir.to_string_lossy().to_string(),
                    blue.to_string_lossy().to_string()
                ],
                cli.output.to_string_lossy().as_ref(),
                !cli.float,
                cli.scale_factor,
                cli.input_scale_factor,
                &cli.compress,          
                cli.compress_level,     
                cli.tiled,          
            )?;
        },
        Commands::Msavi2 { nir, red } => {
            // Create MSAVI2 calculator with NIR and Red bands
            let msavi2 = MSAVI2::new(0, 1, None);
            
            processor.process(
                msavi2,
                &[
                    nir.to_string_lossy().to_string(),
                    red.to_string_lossy().to_string()
                ],
                cli.output.to_string_lossy().as_ref(),
                !cli.float,
                cli.scale_factor,
                cli.input_scale_factor,
                &cli.compress,          
                cli.compress_level,     
                cli.tiled,          
            )?;
        },
        
        Commands::Osavi { nir, red } => {
            // Create OSAVI calculator with NIR and Red bands
            let osavi = OSAVI::new(0, 1, None);
            
            processor.process(
                osavi,
                &[
                    nir.to_string_lossy().to_string(),
                    red.to_string_lossy().to_string()
                ],
                cli.output.to_string_lossy().as_ref(),
                !cli.float,
                cli.scale_factor,
                cli.input_scale_factor,
                &cli.compress,          
                cli.compress_level,     
                cli.tiled,          
            )?;
        }
    }

    println!("Processing complete: {}", cli.output.display());
    Ok(())
}