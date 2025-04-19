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

## Features

- Calculate common vegetation indices (NDVI, EVI, SAVI)
- Water indices (NDWI, MNDWI) 
- Soil and mineral indices (NDSI, BSI)
- Custom index formulas via simple expression syntax
- Parallel processing for large rasters
- Memory-efficient streaming for massive files
- Fixed-point arithmetic optimization (smaller outputs, faster processing)
- Support for GeoTIFF, JPEG2000, and other GDAL formats

## Installation

```bash
# From source
git clone https://github.com/ominiverdi/raster-calc.git
cd raster-calc
cargo build --release

# Copy to your path
cp target/release/raster-calc ~/.local/bin/
```

## Usage

```bash
# Calculate NDVI with float output
raster-calc ndi -a NIR_BAND.tif -b RED_BAND.tif -o output.tif --float

# Calculate NDVI with int16 output (more efficient)
raster-calc ndi -a NIR_BAND.tif -b RED_BAND.tif -o output.tif

# For Sentinel-2 specifically:
raster-calc ndi -a B08.jp2 -b B04.jp2 -o ndvi.tif
```

## Equivalent gdal_calc.py Commands

For comparison, the equivalent gdal_calc.py commands are:

```bash
# Float32 output
gdal_calc.py --calc="(A.astype(float)-1000)/(10000.0)-(B.astype(float)-1000)/(10000.0))/((A.astype(float)-1000)/(10000.0)+(B.astype(float)-1000)/(10000.0))" \
  -A NIR_BAND.jp2 -B RED_BAND.jp2 --outfile=ndvi.tif \
  --type=Float32 --NoDataValue=-999 --co="COMPRESS=DEFLATE" --co="TILED=YES"

# Int16 output  
gdal_calc.py --calc="numpy.int16(((A.astype(float)-1000)/(10000.0)-(B.astype(float)-1000)/(10000.0))/((A.astype(float)-1000)/(10000.0)+(B.astype(float)-1000)/(10000.0)) * 10000)" \
  -A NIR_BAND.jp2 -B RED_BAND.jp2 --outfile=ndvi.tif \
  --type=Int16 --NoDataValue=-10000 --co="COMPRESS=DEFLATE" --co="TILED=YES"
```

## Why raster-calc is faster

- Parallel chunk-based processing using Rayon
- Optimized memory handling with reusable buffers
- Fixed-point arithmetic for int16 output
- GDAL read/write optimizations

## Implementation Details

raster-calc takes advantage of Rust's performance characteristics:

1. **Efficient Memory Management**: Zero-copy data handling where possible
2. **Parallel I/O**: Multiple threads for simultaneous read/write operations
3. **SIMD Optimizations**: Automatically applied by the Rust compiler
4. **Chunk-based Processing**: Cache-friendly algorithms for large images

## Command-line Options

```
USAGE:
    raster-calc [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -o, --output <FILE>             Output file path [default: output.tif]
    --float                         Use float32 instead of int16
    --scale-factor <VALUE>          Scaling factor for fixed-point [default: 10000]
    -h, --help                      Print help information
    -V, --version                   Print version information

SUBCOMMANDS:
    ndi                             Normalized Difference Index: (A-B)/(A+B)
    evi                             Enhanced Vegetation Index
    savi                            Soil Adjusted Vegetation Index
    help                            Print this message or help for a subcommand
```

## Author

Lorenzo Becchi

## Acknowledgments

Special thanks to GrayShade for contributions and optimizations.

## License

MIT License - See LICENSE file for details