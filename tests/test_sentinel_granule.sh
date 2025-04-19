#!/bin/bash
# tests/test_sentinel_granule.sh
# Enhanced version to test all supported indices

# Set execution to exit on error
set -e

SENTINEL_DATA_DIR="../../spectral-calc-tests/data"
OUTPUT_DIR="../data"

# Create output directory if it doesn't exist
mkdir -p $OUTPUT_DIR

echo "Starting spectral indices tests with Sentinel-2 data..."

# Check if sentinel data exists
if [ ! -f "${SENTINEL_DATA_DIR}/T33TTG_20250305T100029_B08_10m.jp2" ]; then
    echo "Error: Sentinel data not found in ${SENTINEL_DATA_DIR}"
    echo "Please make sure the data is available or update the path"
    exit 1
fi

# Identify available bands
echo "Checking available Sentinel-2 bands..."
BAND_NIR="${SENTINEL_DATA_DIR}/T33TTG_20250305T100029_B08_10m.jp2"
BAND_RED="${SENTINEL_DATA_DIR}/T33TTG_20250305T100029_B04_10m.jp2"
BAND_GREEN="${SENTINEL_DATA_DIR}/T33TTG_20250305T100029_B03_10m.jp2"
BAND_BLUE="${SENTINEL_DATA_DIR}/T33TTG_20250305T100029_B02_10m.jp2"

# Check bands existence
[ -f "$BAND_NIR" ] && echo "✓ Found NIR band (B08)" || { echo "✗ NIR band not found!"; BAND_NIR=""; }
[ -f "$BAND_RED" ] && echo "✓ Found RED band (B04)" || { echo "✗ RED band not found!"; BAND_RED=""; }
[ -f "$BAND_GREEN" ] && echo "✓ Found GREEN band (B03)" || { echo "✗ GREEN band not found!"; BAND_GREEN=""; }
[ -f "$BAND_BLUE" ] && echo "✓ Found BLUE band (B02)" || { echo "✗ BLUE band not found!"; BAND_BLUE=""; }

# NDVI tests (float32 and int16)
if [ -n "$BAND_NIR" ] && [ -n "$BAND_RED" ]; then
    echo -e "\nTesting NDVI (float32)..."
    time ../target/release/raster-calc ndi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      -o "${OUTPUT_DIR}/ndvi_large_float.tif" --float

    echo -e "\nTesting NDVI (int16)..."
    time ../target/release/raster-calc ndi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      -o "${OUTPUT_DIR}/ndvi_large_int16.tif"
else
    echo "Skipping NDVI tests - required bands not available"
fi

# EVI tests (float32 and int16)
if [ -n "$BAND_NIR" ] && [ -n "$BAND_RED" ] && [ -n "$BAND_BLUE" ]; then
    echo -e "\nTesting EVI (float32)..."
    time ../target/release/raster-calc evi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      -c "$BAND_BLUE" \
      -o "${OUTPUT_DIR}/evi_large_float.tif" --float

    echo -e "\nTesting EVI (int16)..."
    time ../target/release/raster-calc evi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      -c "$BAND_BLUE" \
      -o "${OUTPUT_DIR}/evi_large_int16.tif"
else
    echo "Skipping EVI tests - required bands not available"
fi

# SAVI tests with different soil factors (float32 and int16)
if [ -n "$BAND_NIR" ] && [ -n "$BAND_RED" ]; then
    echo -e "\nTesting SAVI with default soil factor (float32)..."
    time ../target/release/raster-calc savi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      -o "${OUTPUT_DIR}/savi_default_float.tif" --float

    echo -e "\nTesting SAVI with default soil factor (int16)..."
    time ../target/release/raster-calc savi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      -o "${OUTPUT_DIR}/savi_default_int16.tif"

    echo -e "\nTesting SAVI with high soil factor (L=1.0, float32)..."
    time ../target/release/raster-calc savi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      -l 1.0 \
      -o "${OUTPUT_DIR}/savi_high_float.tif" --float

    echo -e "\nTesting SAVI with low soil factor (L=0.25, int16)..."
    time ../target/release/raster-calc savi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      -l 0.25 \
      -o "${OUTPUT_DIR}/savi_low_int16.tif"
else
    echo "Skipping SAVI tests - required bands not available"
fi

# NDWI tests (float32 and int16)
if [ -n "$BAND_GREEN" ] && [ -n "$BAND_NIR" ]; then
    echo -e "\nTesting NDWI (float32)..."
    time ../target/release/raster-calc ndwi \
      -a "$BAND_GREEN" \
      -b "$BAND_NIR" \
      -o "${OUTPUT_DIR}/ndwi_large_float.tif" --float

    echo -e "\nTesting NDWI (int16)..."
    time ../target/release/raster-calc ndwi \
      -a "$BAND_GREEN" \
      -b "$BAND_NIR" \
      -o "${OUTPUT_DIR}/ndwi_large_int16.tif"
else
    echo "Skipping NDWI tests - required bands not available"
fi

# Verify outputs exist and show basic info
echo -e "\nVerifying outputs:"
echo "===================="
for file in ${OUTPUT_DIR}/*large*.tif; do
    if [ -f "$file" ]; then
        echo "✓ $file"
        echo "  Size: $(du -h $file | cut -f1)"
        gdalinfo -stats $file | grep -E "Size is|Min=|Max=|Mean=|StdDev=" | sed 's/^/  /'
        echo ""
    fi
done

echo "Tests completed!"