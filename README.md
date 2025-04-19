# raster-calc

A high-performance command-line tool for calculating spectral indices from multi-band satellite imagery and remote sensing data. Built in Rust for exceptional speed and memory efficiency.

## Features
- Calculate common vegetation indices (NDVI, EVI, SAVI)
- Water indices (NDWI, MNDWI)
- Soil and mineral indices (NDSI, BSI)
- Custom index formulas via simple expression syntax
- Parallel processing for large rasters
- Memory-efficient streaming for massive files
- Fixed-point arithmetic optimization
- Support for GeoTIFF, JPEG2000, and other GDAL formats

Perfect for processing Sentinel-2, Landsat, and other multi-spectral satellite imagery.

## Example
```
echo "Testing with float output:"
time ../target/release/raster-calc ndi \
  -a ../../spectral-calc-tests/data/T33TTG_20250305T100029_B08_10m.jp2 \
  -b ../../spectral-calc-tests/data/T33TTG_20250305T100029_B04_10m.jp2 \
  -o ../data/ndvi_large_float.tif --float

echo "Testing with fixed-point output:"
time ../target/release/raster-calc ndi \
  -a ../../spectral-calc-tests/data/T33TTG_20250305T100029_B08_10m.jp2 \
  -b ../../spectral-calc-tests/data/T33TTG_20250305T100029_B04_10m.jp2 \
  -o ../data/ndvi_large_int16.tif
  ```