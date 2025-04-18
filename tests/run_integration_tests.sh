#!/bin/bash
# tests/run_integration_tests.sh

set -e

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
echo "Comparing with reference output..."
gdalinfo -stats data/ndvi_reference.tif | grep "Min/Max/Mean/StdDev"
gdalinfo -stats data/ndvi_output_float.tif | grep "Min/Max/Mean/StdDev"

echo "All tests completed successfully!"