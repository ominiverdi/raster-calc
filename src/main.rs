// src/main.rs
use anyhow::Result;
use clap::Parser;

mod cli;
mod indices;
mod io;
mod utils;

use crate::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Ndi { band_a, band_b } => {
            indices::ndi::calculate_ndi(
                band_a.as_path(),
                band_b.as_path(),
                &cli.output,
                !cli.float,
                cli.scale_factor,
            )?;
        }
        &Commands::Evi { .. } | &Commands::Savi { .. } => todo!(),
    }

    println!("Processing complete: {}", cli.output.display());

    Ok(())
}
