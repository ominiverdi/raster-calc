// src/main.rs
use anyhow::Result;
use clap::Parser;

mod cli;
mod io;
mod utils;
mod processing;

use crate::cli::{Cli, Commands};
use crate::processing::{ParallelProcessor, indices::NDI};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let processor = ParallelProcessor::new(None);

    match &cli.command {
        Commands::Ndi { band_a, band_b } => {
            // Create NDI calculator with band_a as first and band_b as second
            let ndi = NDI::new(0, 1, None);
            
            processor.process(
                ndi,
                &[band_a.to_string_lossy().to_string(), band_b.to_string_lossy().to_string()],
                cli.output.to_string_lossy().as_ref(),
                !cli.float,
                cli.scale_factor,
            )?;
        }
        &Commands::Evi { .. } | &Commands::Savi { .. } => todo!(),
    }

    println!("Processing complete: {}", cli.output.display());
    Ok(())
}