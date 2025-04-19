#!/bin/bash
# tests/run_integration_tests.sh

set -e

# tell GDAL to be maximally verbose
# export CPL_DEBUG=ON

echo "Running raster-calc integration tests..."

# Build latest version
cd ..
cargo build --release

# Test NDI calculation (float output)
echo "Testing NDI with float output..."
./target/release/raster-calc ndi -a data/nir.tif -b data/red.tif -o data/ndvi_output_float.tif --float

# Test NDI calculation (fixed-point output)
echo "Testing NDI with fixed-point output..."
./target/release/raster-calc ndi -a data/nir.tif -b data/red.tif -o data/ndvi_output_int16.tif

# Test EVI calculation
echo "Testing EVI calculation..."
./target/release/raster-calc evi -a data/nir.tif -b data/red.tif -c data/blue.tif -o data/evi_output.tif

# Verify outputs exist
for file in data/ndvi_output_float.tif data/ndvi_output_int16.tif data/evi_output.tif; do
    if [ ! -f "$file" ]; then
        echo "ERROR: Output file $file not created!"
        exit 1
    fi
    echo "✓ $file created successfully"
done

# Compare with reference (basic stats check)
echo "Comparing outputs..."
echo "Reference NDVI:"
gdalinfo data/ndvi_reference.tif | grep "Size is"
echo "Output NDVI (float):"
gdalinfo data/ndvi_output_float.tif | grep "Size is"

echo "All tests completed successfully!"




## local test
# raster-calc ndi -a <first_band> -b <second_band> -o <output>

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