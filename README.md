# raster-calc

A high-performance command-line tool for calculating spectral indices from multi-band satellite imagery and remote sensing data. Built in Rust for exceptional speed and memory efficiency.

## Performance

raster-calc delivers significantly faster processing compared to traditional GDAL utilities:

| Implementation | NDVI (float32) | NDVI (int16) |
|----------------|---------------|--------------|
| raster-calc    | 2.86s         | 2.69s        |
| gdal_calc.py   | 12.47s        | 6.97s        |
| **Speedup**    | **4.4x**      | **2.6x**     |

*Benchmark on Intel i9-10900 with Sentinel-2 10980×10980 image (10m resolution)*

## Performance Optimizations

raster-calc includes several performance optimizations:

1. **Dataset Caching**: Automatically caches datasets across operations, reducing redundant file operations (up to 56% speedup)
2. **Parallel Processing**: Efficiently utilizes multiple CPU cores to process operations in parallel (up to 24% additional speedup)
3. **Thread Pool Tuning**: Optimizes thread usage based on system capabilities (up to 5% additional speedup)

These optimizations combine to provide up to 3.5x faster processing compared to naive implementations.

## Supported Indices

| Index | Name | Formula | Application |
|-------|------|---------|-------------|
| NDI/NDVI | Normalized Difference (Vegetation) Index | (NIR-RED)/(NIR+RED) | Vegetation health monitoring |
| EVI | Enhanced Vegetation Index | 2.5 × (NIR-RED)/(NIR+6×RED-7.5×BLUE+1) | Improved vegetation monitoring in high biomass regions |
| SAVI | Soil Adjusted Vegetation Index | [(NIR-RED)/(NIR+RED+L)] × (1+L) | Vegetation in areas with high soil exposure |
| NDWI | Normalized Difference Water Index | (GREEN-NIR)/(GREEN+NIR) | Surface water detection |
| NDSI | Normalized Difference Snow Index | (GREEN-SWIR)/(GREEN+SWIR) | Snow and ice detection |
| BSI | Bare Soil Index | [(SWIR+RED)-(NIR+BLUE)]/[(SWIR+RED)+(NIR+BLUE)] | Bare soil and urban areas |
| MSAVI2 | Modified Soil Adjusted Vegetation Index | [2×NIR+1-√((2×NIR+1)²-8×(NIR-RED))]/2 | Improved correction for soil influence |
| OSAVI | Optimized Soil Adjusted Vegetation Index | (NIR-RED)/(NIR+RED+0.16) × 1.16 | Optimized for agricultural monitoring |

## Features

- Calculate common vegetation indices (NDVI, EVI, SAVI, MSAVI2, OSAVI)
- Water indices (NDWI)
- Snow and ice indices (NDSI)
- Soil and mineral indices (BSI)
- Parallel processing for large rasters
- Memory-efficient streaming for massive files
- Fixed-point arithmetic optimization (smaller outputs, faster processing)
- Support for GeoTIFF, JPEG2000, and other GDAL formats
- Customizable compression options
- Batch processing for multiple operations

## Installation

```bash
# From source
git clone https://github.com/ominiverdi/raster-calc.git
cd raster-calc
cargo build --release

# Copy to your path
cp target/release/raster-calc ~/.local/bin/
```

## Usage Examples

```bash
# Calculate NDVI (Normalized Difference Vegetation Index)
raster-calc ndi -a NIR_BAND.tif -b RED_BAND.tif -o ndvi_output.tif

# Calculate EVI (Enhanced Vegetation Index)
raster-calc evi -a NIR_BAND.tif -b RED_BAND.tif -c BLUE_BAND.tif -o evi_output.tif

# Calculate SAVI (Soil Adjusted Vegetation Index) with custom soil factor
raster-calc savi -a NIR_BAND.tif -b RED_BAND.tif -l 0.8 -o savi_output.tif

# Calculate NDWI (Normalized Difference Water Index)
raster-calc ndwi -a GREEN_BAND.tif -b NIR_BAND.tif -o ndwi_output.tif

# Calculate NDSI (Normalized Difference Snow Index)
raster-calc ndsi -a GREEN_BAND.tif -b SWIR_BAND.tif -o ndsi_output.tif

# Calculate BSI (Bare Soil Index)
raster-calc bsi -s SWIR_BAND.tif -r RED_BAND.tif -n NIR_BAND.tif -b BLUE_BAND.tif -o bsi_output.tif

# Calculate MSAVI2 (Modified Soil Adjusted Vegetation Index)
raster-calc msavi2 -a NIR_BAND.tif -b RED_BAND.tif -o msavi2_output.tif

# Calculate OSAVI (Optimized Soil Adjusted Vegetation Index)
raster-calc osavi -a NIR_BAND.tif -b RED_BAND.tif -o osavi_output.tif

# Batch Processing
raster-calc batch --config batch_config.json
# Short form
raster-calc batch -c batch_config.json

# For Sentinel-2 specifically:
raster-calc ndi -a B08.jp2 -b B04.jp2 -o ndvi.tif
```

## Batch Processing

For processing multiple operations in a single run, use the batch processing feature:

```json
{
  "global": {
    "compress": "DEFLATE",
    "compress_level": 6,
    "float": true,
    "scale_factor": 10000,
    "tiled": true,
    "threads": 12
  },
  "operations": [
    {
      "type": "ndi",
      "params": { "a": "NIR_BAND.tif", "b": "RED_BAND.tif" },
      "output": "ndvi_output.tif"
    },
    {
      "type": "evi",
      "params": { "a": "NIR_BAND.tif", "b": "RED_BAND.tif", "c": "BLUE_BAND.tif" },
      "output": "evi_output.tif",
      "float": false
    }
  ]
}
```

### Parallelization Settings

The `threads` parameter in the global section controls how many operations are processed in parallel:

```json
"global": {
  "threads": 12,
  // other settings
}
```

#### Thread Count Recommendations:

| System Type | CPU Cores | RAM | Recommended Threads |
|-------------|-----------|-----|---------------------|
| Low-end     | 2-4 cores | 4-8GB | 2-3 |
| Mid-range   | 6-8 cores | 16GB | 4-6 |
| High-end    | 12-16 cores | 32GB+ | 8-12 |
| Workstation | 16+ cores | 64GB+ | 12-16 |

**Finding the optimal thread count:**
* Too few threads: Underutilizes your system
* Too many threads: Creates I/O contention and reduces performance
* Optimal settings depend on your specific hardware, especially storage speed

Our benchmarks show that for a high-end system with 20 threads and 64GB RAM, **12 threads** provides the optimal performance balance. Increasing beyond this can actually decrease performance due to I/O bottlenecks.

If no thread count is specified, raster-calc automatically calculates an appropriate value based on your system's available CPU cores.

## Compression Options

raster-calc supports various compression options for the output GeoTIFF files:

```bash
# Specify compression algorithm (NONE, DEFLATE, LZW, ZSTD)
raster-calc ndi -a NIR.tif -b RED.tif -o output.tif --compress ZSTD

# Set compression level (1-9 for DEFLATE, 1-22 for ZSTD)
raster-calc ndi -a NIR.tif -b RED.tif -o output.tif --compress ZSTD --compress-level 12

# Disable tiled output for special use cases
raster-calc ndi -a NIR.tif -b RED.tif -o output.tif --tiled false
```

Default settings:
- Compression: DEFLATE
- Compression Level: 6
- Tiled: true
- Threads: Auto-detected based on system

## Why raster-calc is faster

- Parallel chunk-based processing using Rayon
- Optimized memory handling with reusable buffers
- Dataset caching to avoid repeated file opening
- Fixed-point arithmetic for int16 output
- GDAL read/write optimizations

## Implementation Details

raster-calc takes advantage of Rust's performance characteristics:

1. **Efficient Memory Management**: Zero-copy data handling where possible
2. **Parallel I/O**: Multiple threads for simultaneous read/write operations
3. **SIMD Optimizations**: Automatically applied by the Rust compiler
4. **Chunk-based Processing**: Cache-friendly algorithms for large images
5. **Dataset Caching**: Eliminates redundant file operations in batch processing

## Command-line Options

```
USAGE:
    raster-calc [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -o, --output <FILE>             Output file path [default: output.tif]
    --float                         Use float32 instead of int16
    --scale-factor <VALUE>          Scaling factor for fixed-point [default: 10000]
    --compress <TYPE>               Compression type: NONE, DEFLATE, LZW, ZSTD [default: DEFLATE]
    --compress-level <LEVEL>        Compression level (1-9 for DEFLATE, 1-22 for ZSTD) [default: 6]
    --tiled <BOOL>                  Use tiled output [default: true]
    -h, --help                      Print help information
    -V, --version                   Print version information

SUBCOMMANDS:
    ndi                             Normalized Difference Index: (A-B)/(A+B)
    evi                             Enhanced Vegetation Index
    savi                            Soil Adjusted Vegetation Index
    ndwi                            Normalized Difference Water Index
    ndsi                            Normalized Difference Snow Index
    batch                           Batch process multiple operations from a JSON config file
        -c, --config <FILE>         Path to the JSON configuration file
    bsi                             Bare Soil Index
    msavi2                          Modified Soil Adjusted Vegetation Index
    osavi                           Optimized Soil Adjusted Vegetation Index
    help                            Print this message or help for a subcommand
```

## Author

Lorenzo Becchi

## License

MIT License - See LICENSE file for details