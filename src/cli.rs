use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "raster-calc")]
#[command(about = "High-performance spectral index calculator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output file path
    #[arg(short, long, default_value = "output.tif", global = true)]
    pub output: PathBuf,

    /// Use float32 instead of int16
    #[arg(long, global = true)]
    pub float: bool,

    /// Scaling factor for fixed-point output
    #[arg(long, default_value = "10000", global = true)]
    pub scale_factor: i32,

    /// Input scaling factor (for L2A data use 10000, for TOA use 1)
    #[arg(long, default_value = "1.0", global = true)]
    pub input_scale_factor: f32,

    /// Compression type (NONE, DEFLATE, LZW, ZSTD)
    #[arg(long, default_value = "DEFLATE", global = true)]
    pub compress: String,

    /// Compression level (1-9 for DEFLATE, 1-22 for ZSTD)
    #[arg(long, default_value = "6", global = true)]
    pub compress_level: u8,

    /// Use tiled output
    #[arg(long, default_value = "true", global = true)]
    pub tiled: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Normalized Difference Index: (A-B)/(A+B) [Scaling: Not needed]
    Ndi {
        /// First band (A)
        #[arg(short = 'a', long)]
        band_a: PathBuf,

        /// Second band (B)
        #[arg(short = 'b', long)]
        band_b: PathBuf,
    },

    /// Enhanced Vegetation Index [Scaling: Required for L2A data]
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

    /// Soil Adjusted Vegetation Index [Scaling: Required for L2A data]
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

    /// Normalized Difference Water Index: (GREEN-NIR)/(GREEN+NIR) [Scaling: Not needed]
    Ndwi {
        /// Green band (A)
        #[arg(short = 'a', long)]
        green: PathBuf,

        /// NIR band (B)
        #[arg(short = 'b', long)]
        nir: PathBuf,
    },

    /// Normalized Difference Snow Index: (GREEN-SWIR)/(GREEN+SWIR) [Scaling: Not needed]
    Ndsi {
        /// Green band (A)
        #[arg(short = 'a', long)]
        green: PathBuf,

        /// SWIR band (B) - typically Sentinel-2 B11
        #[arg(short = 'b', long)]
        swir: PathBuf,
    },

    /// Bare Soil Index: ((SWIR+RED)-(NIR+BLUE))/((SWIR+RED)+(NIR+BLUE)) [Scaling: Not needed]
    Bsi {
        /// SWIR band - typically Sentinel-2 B11
        #[arg(short = 's', long)]
        swir: PathBuf,

        /// RED band
        #[arg(short = 'r', long)]
        red: PathBuf,

        /// NIR band
        #[arg(short = 'n', long)]
        nir: PathBuf,

        /// BLUE band
        #[arg(short = 'b', long)]
        blue: PathBuf,
    },

    /// Modified Soil Adjusted Vegetation Index 2 [Scaling: Required for L2A data]
    Msavi2 {
        /// NIR band
        #[arg(short = 'a', long)]
        nir: PathBuf,

        /// Red band
        #[arg(short = 'b', long)]
        red: PathBuf,
    },

    /// Optimized Soil Adjusted Vegetation Index [Scaling: Required for L2A data]
    Osavi {
        /// NIR band
        #[arg(short = 'a', long)]
        nir: PathBuf,

        /// Red band
        #[arg(short = 'b', long)]
        red: PathBuf,
    },

    /// Process multiple operations from a JSON configuration file
    Batch {
        /// Path to JSON configuration file
        #[arg(short = 'c', long)]
        config: PathBuf,
    },
}