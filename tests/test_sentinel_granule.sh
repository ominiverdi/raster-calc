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
BAND_SWIR="${SENTINEL_DATA_DIR}/T33TTG_20250305T100029_B11_20m.jp2"

# Check bands existence
[ -f "$BAND_NIR" ] && echo "✓ Found NIR band (B08)" || { echo "✗ NIR band not found!"; BAND_NIR=""; }
[ -f "$BAND_RED" ] && echo "✓ Found RED band (B04)" || { echo "✗ RED band not found!"; BAND_RED=""; }
[ -f "$BAND_GREEN" ] && echo "✓ Found GREEN band (B03)" || { echo "✗ GREEN band not found!"; BAND_GREEN=""; }
[ -f "$BAND_BLUE" ] && echo "✓ Found BLUE band (B02)" || { echo "✗ BLUE band not found!"; BAND_BLUE=""; }
[ -f "$BAND_SWIR" ] && echo "✓ Found SWIR band (B11)" || { echo "✗ SWIR band not found!"; BAND_SWIR=""; }

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

# NDSI tests (float32 and int16) - FIXED: Use same resolution bands
if [ -n "$BAND_GREEN" ] && [ -n "$BAND_SWIR" ]; then
    # Use 20m resolution for both bands to avoid size mismatch
    BAND_GREEN_20M="${SENTINEL_DATA_DIR}/T33TTG_20250305T100029_B03_20m.jp2"
    
    if [ -f "$BAND_GREEN_20M" ]; then
        echo -e "\nTesting NDSI (float32)..."
        time ../target/release/raster-calc ndsi \
          -a "$BAND_GREEN_20M" \
          -b "$BAND_SWIR" \
          -o "${OUTPUT_DIR}/ndsi_large_float.tif" --float

        echo -e "\nTesting NDSI (int16)..."
        time ../target/release/raster-calc ndsi \
          -a "$BAND_GREEN_20M" \
          -b "$BAND_SWIR" \
          -o "${OUTPUT_DIR}/ndsi_large_int16.tif"
    else
        echo "Skipping NDSI tests - B03_20m not available"
    fi
else
    echo "Skipping NDSI tests - required bands not available"
fi

# Input scaling tests for indices that need it
echo -e "\n=== INPUT SCALING TESTS ==="

# EVI with input scaling (L2A correction)
if [ -n "$BAND_NIR" ] && [ -n "$BAND_RED" ] && [ -n "$BAND_BLUE" ]; then
    echo -e "\nTesting EVI with L2A input scaling (float32)..."
    time ../target/release/raster-calc evi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      -c "$BAND_BLUE" \
      --input-scale-factor 10000 \
      -o "${OUTPUT_DIR}/evi_scaled_float.tif" --float

    echo -e "\nTesting EVI with L2A input scaling (int16)..."
    time ../target/release/raster-calc evi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      -c "$BAND_BLUE" \
      --input-scale-factor 10000 \
      -o "${OUTPUT_DIR}/evi_scaled_int16.tif"
else
    echo "Skipping EVI scaling tests - required bands not available"
fi

# SAVI with input scaling (L2A correction)
if [ -n "$BAND_NIR" ] && [ -n "$BAND_RED" ]; then
    echo -e "\nTesting SAVI with L2A input scaling (float32)..."
    time ../target/release/raster-calc savi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      --input-scale-factor 10000 \
      -o "${OUTPUT_DIR}/savi_scaled_float.tif" --float

    echo -e "\nTesting SAVI with L2A input scaling (int16)..."
    time ../target/release/raster-calc savi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      --input-scale-factor 10000 \
      -o "${OUTPUT_DIR}/savi_scaled_int16.tif"
else
    echo "Skipping SAVI scaling tests - required bands not available"
fi

# MSAVI2 with input scaling (L2A correction)
if [ -n "$BAND_NIR" ] && [ -n "$BAND_RED" ]; then
    echo -e "\nTesting MSAVI2 with L2A input scaling (float32)..."
    time ../target/release/raster-calc msavi2 \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      --input-scale-factor 10000 \
      -o "${OUTPUT_DIR}/msavi2_scaled_float.tif" --float

    echo -e "\nTesting MSAVI2 with L2A input scaling (int16)..."
    time ../target/release/raster-calc msavi2 \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      --input-scale-factor 10000 \
      -o "${OUTPUT_DIR}/msavi2_scaled_int16.tif"
else
    echo "Skipping MSAVI2 scaling tests - required bands not available"
fi

# OSAVI with input scaling (L2A correction)
if [ -n "$BAND_NIR" ] && [ -n "$BAND_RED" ]; then
    echo -e "\nTesting OSAVI with L2A input scaling (float32)..."
    time ../target/release/raster-calc osavi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      --input-scale-factor 10000 \
      -o "${OUTPUT_DIR}/osavi_scaled_float.tif" --float

    echo -e "\nTesting OSAVI with L2A input scaling (int16)..."
    time ../target/release/raster-calc osavi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      --input-scale-factor 10000 \
      -o "${OUTPUT_DIR}/osavi_scaled_int16.tif"
else
    echo "Skipping OSAVI scaling tests - required bands not available"
fi

# NDI scaling test (should give identical results with/without scaling)
if [ -n "$BAND_NIR" ] && [ -n "$BAND_RED" ]; then
    echo -e "\nTesting NDI scaling verification (should be identical to regular NDVI)..."
    time ../target/release/raster-calc ndi \
      -a "$BAND_NIR" \
      -b "$BAND_RED" \
      --input-scale-factor 10000 \
      -o "${OUTPUT_DIR}/ndvi_scaling_test.tif" --float
else
    echo "Skipping NDI scaling test - required bands not available"
fi

# BSI tests (float32 and int16)
if [ -n "$BAND_SWIR" ] && [ -n "$BAND_RED" ] && [ -n "$BAND_NIR" ] && [ -n "$BAND_BLUE" ]; then
    echo -e "\nTesting BSI (float32)..."
    time ../target/release/raster-calc bsi \
      -s "$BAND_SWIR" \
      -r "$BAND_RED" \
      -n "$BAND_NIR" \
      -b "$BAND_BLUE" \
      -o "${OUTPUT_DIR}/bsi_large_float.tif" --float

    echo -e "\nTesting BSI (int16)..."
    time ../target/release/raster-calc bsi \
      -s "$BAND_SWIR" \
      -r "$BAND_RED" \
      -n "$BAND_NIR" \
      -b "$BAND_BLUE" \
      -o "${OUTPUT_DIR}/bsi_large_int16.tif"
else
    echo "Skipping BSI tests - required bands not available"
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