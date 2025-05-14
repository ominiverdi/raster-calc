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

# For Sentinel-2 specifically:
raster-calc ndi -a B08.jp2 -b B04.jp2 -o ndvi.tif
```

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
- Threads: All available CPUs

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
    bsi                             Bare Soil Index
    msavi2                          Modified Soil Adjusted Vegetation Index
    osavi                           Optimized Soil Adjusted Vegetation Index
    help                            Print this message or help for a subcommand
```

## Author

Lorenzo Becchi

## Acknowledgments

Special thanks to [GrayShade/lnicola](https://github.com/lnicola) for contributions and optimizations.

## License

MIT License - See LICENSE file for details