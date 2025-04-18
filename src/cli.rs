use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "raster-calc")]
#[command(about = "High-performance spectral index calculator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Output file path
    #[arg(short, long, default_value = "output.tif")]
    pub output: PathBuf,
    
    /// Use float32 instead of int16
    #[arg(long)]
    pub float: bool,
    
    /// Scaling factor for fixed-point
    #[arg(long, default_value = "10000")]
    pub scale_factor: i32,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Normalized Difference Index: (A-B)/(A+B)
    Ndi {
        /// First band (A)
        #[arg(short = 'a', long)]
        band_a: PathBuf,
        
        /// Second band (B)
        #[arg(short = 'b', long)]
        band_b: PathBuf,
    },
    
    /// Enhanced Vegetation Index
    Evi {
        /// NIR band (A)
        #[arg(short = 'a', long)]
        nir: PathBuf,
        
        /// Red band (B)
        #[arg(short = 'b', long)]
        red: PathBuf,
        
        /// Blue band (C)
        #[arg(short = 'c', long)]
        blue: PathBuf,
    },
    /// Soil Adjusted Vegetation Index
    Savi {
        /// NIR band (A)
        #[arg(short = 'a', long)]
        nir: PathBuf,
        
        /// Red band (B)
        #[arg(short = 'b', long)]
        red: PathBuf,
        
        /// Soil adjustment factor (default: 0.5)
        #[arg(short = 'l', long, default_value = "0.5")]
        soil_factor: f32,
    },
}