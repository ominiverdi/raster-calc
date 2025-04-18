#!/bin/bash
# tests/generate_test_data.sh

set -e
echo "Generating synthetic test data for raster-calc..."

# Create test data directory
mkdir -p ../data

# Size of test rasters (small for quick tests)
SIZE=256

# Create template raster using gdal_create
echo "Creating synthetic rasters..."
gdal_create -of GTiff -outsize $SIZE $SIZE -bands 1 -ot Float32 ../data/template.tif

# Use gdal_calc with simple calculations and overwrite flag
gdal_calc.py --calc="5000" --outfile=../data/nir.tif -A ../data/template.tif --NoDataValue=0 --type=Float32 --overwrite
gdal_calc.py --calc="2500" --outfile=../data/red.tif -A ../data/template.tif --NoDataValue=0 --type=Float32 --overwrite
gdal_calc.py --calc="1500" --outfile=../data/blue.tif -A ../data/template.tif --NoDataValue=0 --type=Float32 --overwrite

# Create reference NDVI using GDAL
echo "Creating reference NDVI..."
gdal_calc.py --calc="(A-B)/(A+B)" --outfile=../data/ndvi_reference.tif \
             -A ../data/nir.tif -B ../data/red.tif --NoDataValue=-999 --type=Float32 --overwrite

echo "Test data generated successfully!"
echo "Available test files:"
ls -lh ../data/