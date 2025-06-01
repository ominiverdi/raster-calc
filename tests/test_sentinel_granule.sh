#!/bin/bash
# tests/test_sentinel_granule.sh
# Enhanced version with comprehensive statistical validation for all spectral indices

# Set execution to exit on error
set -e

SENTINEL_DATA_DIR="../../spectral-calc-tests/data"
OUTPUT_DIR="../data"

# Create output directory if it doesn't exist
mkdir -p $OUTPUT_DIR

echo "=========================================="
echo "Spectral Indices Statistical Validation"
echo "=========================================="

# Define expected ranges for each index
declare -A EXPECTED_RANGES
EXPECTED_RANGES[ndvi]="-1.0 1.0"
EXPECTED_RANGES[evi]="-1.0 1.0"
EXPECTED_RANGES[savi]="-1.0 1.0"
EXPECTED_RANGES[ndwi]="-1.0 1.0"
EXPECTED_RANGES[ndsi]="-1.0 1.0"
EXPECTED_RANGES[bsi]="-1.0 1.0"
EXPECTED_RANGES[msavi2]="0.0 1.0"
EXPECTED_RANGES[osavi]="-1.0 1.0"

# Define typical vegetation/feature ranges
declare -A TYPICAL_RANGES
TYPICAL_RANGES[ndvi]="0.2 0.8"
TYPICAL_RANGES[evi]="0.2 0.8"
TYPICAL_RANGES[savi]="0.2 0.8"
TYPICAL_RANGES[ndwi]="-0.3 0.3"
TYPICAL_RANGES[ndsi]="-0.2 0.8"
TYPICAL_RANGES[bsi]="-0.5 0.5"
TYPICAL_RANGES[msavi2]="0.2 0.8"
TYPICAL_RANGES[osavi]="0.2 0.8"

# Function to extract statistics from gdalinfo output
extract_stats() {
    local file="$1"
    if [ ! -f "$file" ]; then
        echo "N/A N/A N/A N/A"
        return
    fi
    
    local stats=$(gdalinfo -stats "$file" 2>/dev/null | grep -E "Minimum=|Maximum=|Mean=|StdDev=" | head -1)
    if [ -z "$stats" ]; then
        echo "N/A N/A N/A N/A"
        return
    fi
    
    # Extract values using sed
    local min=$(echo "$stats" | sed -n 's/.*Minimum=\([^,]*\).*/\1/p')
    local max=$(echo "$stats" | sed -n 's/.*Maximum=\([^,]*\).*/\1/p')
    local mean=$(echo "$stats" | sed -n 's/.*Mean=\([^,]*\).*/\1/p')
    local stddev=$(echo "$stats" | sed -n 's/.*StdDev=\([^,]*\).*/\1/p')
    
    echo "$min $max $mean $stddev"
}

# Function to validate range
validate_range() {
    local index_name="$1"
    local min_val="$2"
    local max_val="$3"
    local mean_val="$4"
    
    if [ "$min_val" = "N/A" ] || [ "$max_val" = "N/A" ]; then
        echo "❌ NO_DATA"
        return
    fi
    
    # Get expected range
    local expected=(${EXPECTED_RANGES[$index_name]})
    local exp_min=${expected[0]}
    local exp_max=${expected[1]}
    
    # Get typical range  
    local typical=(${TYPICAL_RANGES[$index_name]})
    local typ_min=${typical[0]}
    local typ_max=${typical[1]}
    
    local status="✅"
    local warnings=""
    
    # Check if values are within expected theoretical range
    if (( $(echo "$min_val < $exp_min" | bc -l) )) || (( $(echo "$max_val > $exp_max" | bc -l) )); then
        status="❌"
        warnings="${warnings}OUT_OF_RANGE "
    fi
    
    # Check if mean is within typical range (warning, not error)
    if (( $(echo "$mean_val < $typ_min" | bc -l) )) || (( $(echo "$mean_val > $typ_max" | bc -l) )); then
        if [ "$status" = "✅" ]; then
            status="⚠️ "
        fi
        warnings="${warnings}ATYPICAL_MEAN "
    fi
    
    # Check for suspicious values
    if (( $(echo "$min_val == $max_val" | bc -l) )); then
        status="❌"
        warnings="${warnings}CONSTANT_VALUE "
    fi
    
    echo "$status $warnings"
}

# Function to compare scaled vs unscaled results
compare_scaling_results() {
    local index_name="$1"
    local file1="$2"  # unscaled
    local file2="$3"  # scaled
    
    if [ ! -f "$file1" ] || [ ! -f "$file2" ]; then
        echo "N/A"
        return
    fi
    
    local stats1=($(extract_stats "$file1"))
    local stats2=($(extract_stats "$file2"))
    
    if [ "${stats1[0]}" = "N/A" ] || [ "${stats2[0]}" = "N/A" ]; then
        echo "N/A"
        return
    fi
    
    # For pure ratio indices (NDI, NDWI, NDSI, BSI), results should be identical
    case $index_name in
        ndvi|ndwi|ndsi|bsi)
            local diff_min=$(echo "${stats1[0]} - ${stats2[0]}" | bc -l | sed 's/-//')
            local diff_max=$(echo "${stats1[1]} - ${stats2[1]}" | bc -l | sed 's/-//')
            
            if (( $(echo "$diff_min < 0.001" | bc -l) )) && (( $(echo "$diff_max < 0.001" | bc -l) )); then
                echo "✅ IDENTICAL"
            else
                echo "❌ DIFFERENT"
            fi
            ;;
        *)
            echo "✅ EXPECTED_DIFF"
            ;;
    esac
}

# Storage for all results
declare -A RESULTS_FLOAT
declare -A RESULTS_INT16
declare -A RESULTS_SCALED_FLOAT
declare -A RESULTS_SCALED_INT16

# Check if sentinel data exists
if [ ! -f "${SENTINEL_DATA_DIR}/T33TTG_20250305T100029_B08_10m.jp2" ]; then
    echo "❌ Error: Sentinel data not found in ${SENTINEL_DATA_DIR}"
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
BAND_GREEN_20M="${SENTINEL_DATA_DIR}/T33TTG_20250305T100029_B03_20m.jp2"

# Check bands existence
[ -f "$BAND_NIR" ] && echo "✅ Found NIR band (B08)" || { echo "❌ NIR band not found!"; exit 1; }
[ -f "$BAND_RED" ] && echo "✅ Found RED band (B04)" || { echo "❌ RED band not found!"; exit 1; }
[ -f "$BAND_GREEN" ] && echo "✅ Found GREEN band (B03)" || { echo "❌ GREEN band not found!"; exit 1; }
[ -f "$BAND_BLUE" ] && echo "✅ Found BLUE band (B02)" || { echo "❌ BLUE band not found!"; exit 1; }
[ -f "$BAND_SWIR" ] && echo "✅ Found SWIR band (B11)" || { echo "❌ SWIR band not found!"; exit 1; }

echo ""
echo "=========================================="
echo "Running spectral index calculations..."
echo "=========================================="

# NDVI tests (float32 and int16)
echo "🔹 Testing NDVI..."
../target/release/raster-calc ndi \
  -a "$BAND_NIR" -b "$BAND_RED" \
  -o "${OUTPUT_DIR}/ndvi_float.tif" --float >/dev/null 2>&1

../target/release/raster-calc ndi \
  -a "$BAND_NIR" -b "$BAND_RED" \
  -o "${OUTPUT_DIR}/ndvi_int16.tif" >/dev/null 2>&1

../target/release/raster-calc ndi \
  -a "$BAND_NIR" -b "$BAND_RED" \
  --input-scale-factor 10000 \
  -o "${OUTPUT_DIR}/ndvi_scaled_float.tif" --float >/dev/null 2>&1

# EVI tests
echo "🔹 Testing EVI..."
../target/release/raster-calc evi \
  -a "$BAND_NIR" -b "$BAND_RED" -c "$BAND_BLUE" \
  -o "${OUTPUT_DIR}/evi_float.tif" --float >/dev/null 2>&1

../target/release/raster-calc evi \
  -a "$BAND_NIR" -b "$BAND_RED" -c "$BAND_BLUE" \
  -o "${OUTPUT_DIR}/evi_int16.tif" >/dev/null 2>&1

../target/release/raster-calc evi \
  -a "$BAND_NIR" -b "$BAND_RED" -c "$BAND_BLUE" \
  --input-scale-factor 10000 \
  -o "${OUTPUT_DIR}/evi_scaled_float.tif" --float >/dev/null 2>&1

# SAVI tests
echo "🔹 Testing SAVI..."
../target/release/raster-calc savi \
  -a "$BAND_NIR" -b "$BAND_RED" \
  -o "${OUTPUT_DIR}/savi_float.tif" --float >/dev/null 2>&1

../target/release/raster-calc savi \
  -a "$BAND_NIR" -b "$BAND_RED" \
  -o "${OUTPUT_DIR}/savi_int16.tif" >/dev/null 2>&1

../target/release/raster-calc savi \
  -a "$BAND_NIR" -b "$BAND_RED" \
  --input-scale-factor 10000 \
  -o "${OUTPUT_DIR}/savi_scaled_float.tif" --float >/dev/null 2>&1

# NDWI tests
echo "🔹 Testing NDWI..."
../target/release/raster-calc ndwi \
  -a "$BAND_GREEN" -b "$BAND_NIR" \
  -o "${OUTPUT_DIR}/ndwi_float.tif" --float >/dev/null 2>&1

../target/release/raster-calc ndwi \
  -a "$BAND_GREEN" -b "$BAND_NIR" \
  -o "${OUTPUT_DIR}/ndwi_int16.tif" >/dev/null 2>&1

../target/release/raster-calc ndwi \
  -a "$BAND_GREEN" -b "$BAND_NIR" \
  --input-scale-factor 10000 \
  -o "${OUTPUT_DIR}/ndwi_scaled_float.tif" --float >/dev/null 2>&1

# NDSI tests
echo "🔹 Testing NDSI..."
../target/release/raster-calc ndsi \
  -a "$BAND_GREEN_20M" -b "$BAND_SWIR" \
  -o "${OUTPUT_DIR}/ndsi_float.tif" --float >/dev/null 2>&1

../target/release/raster-calc ndsi \
  -a "$BAND_GREEN_20M" -b "$BAND_SWIR" \
  -o "${OUTPUT_DIR}/ndsi_int16.tif" >/dev/null 2>&1

../target/release/raster-calc ndsi \
  -a "$BAND_GREEN_20M" -b "$BAND_SWIR" \
  --input-scale-factor 10000 \
  -o "${OUTPUT_DIR}/ndsi_scaled_float.tif" --float >/dev/null 2>&1

# BSI tests  
echo "🔹 Testing BSI..."
../target/release/raster-calc bsi \
  -s "$BAND_SWIR" -r "$BAND_RED" -n "$BAND_NIR" -b "$BAND_BLUE" \
  -o "${OUTPUT_DIR}/bsi_float.tif" --float >/dev/null 2>&1

../target/release/raster-calc bsi \
  -s "$BAND_SWIR" -r "$BAND_RED" -n "$BAND_NIR" -b "$BAND_BLUE" \
  -o "${OUTPUT_DIR}/bsi_int16.tif" >/dev/null 2>&1

../target/release/raster-calc bsi \
  -s "$BAND_SWIR" -r "$BAND_RED" -n "$BAND_NIR" -b "$BAND_BLUE" \
  --input-scale-factor 10000 \
  -o "${OUTPUT_DIR}/bsi_scaled_float.tif" --float >/dev/null 2>&1

# MSAVI2 tests
echo "🔹 Testing MSAVI2..."
../target/release/raster-calc msavi2 \
  -a "$BAND_NIR" -b "$BAND_RED" \
  -o "${OUTPUT_DIR}/msavi2_float.tif" --float >/dev/null 2>&1

../target/release/raster-calc msavi2 \
  -a "$BAND_NIR" -b "$BAND_RED" \
  -o "${OUTPUT_DIR}/msavi2_int16.tif" >/dev/null 2>&1

../target/release/raster-calc msavi2 \
  -a "$BAND_NIR" -b "$BAND_RED" \
  --input-scale-factor 10000 \
  -o "${OUTPUT_DIR}/msavi2_scaled_float.tif" --float >/dev/null 2>&1

# OSAVI tests
echo "🔹 Testing OSAVI..."
../target/release/raster-calc osavi \
  -a "$BAND_NIR" -b "$BAND_RED" \
  -o "${OUTPUT_DIR}/osavi_float.tif" --float >/dev/null 2>&1

../target/release/raster-calc osavi \
  -a "$BAND_NIR" -b "$BAND_RED" \
  -o "${OUTPUT_DIR}/osavi_int16.tif" >/dev/null 2>&1

../target/release/raster-calc osavi \
  -a "$BAND_NIR" -b "$BAND_RED" \
  --input-scale-factor 10000 \
  -o "${OUTPUT_DIR}/osavi_scaled_float.tif" --float >/dev/null 2>&1

echo ""
echo "=========================================="
echo "STATISTICAL VALIDATION RESULTS"
echo "=========================================="

# Collect all statistics
for index in ndvi evi savi ndwi ndsi bsi msavi2 osavi; do
    RESULTS_FLOAT[$index]=$(extract_stats "${OUTPUT_DIR}/${index}_float.tif")
    RESULTS_INT16[$index]=$(extract_stats "${OUTPUT_DIR}/${index}_int16.tif")
    RESULTS_SCALED_FLOAT[$index]=$(extract_stats "${OUTPUT_DIR}/${index}_scaled_float.tif")
done

echo ""
printf "%-8s %-6s %-8s %-8s %-8s %-8s %-15s %-15s\n" \
    "INDEX" "TYPE" "MIN" "MAX" "MEAN" "STDDEV" "VALIDATION" "SCALING_CHECK"
printf "%-8s %-6s %-8s %-8s %-8s %-8s %-15s %-15s\n" \
    "────────" "──────" "────────" "────────" "────────" "────────" "───────────────" "───────────────"

for index in ndvi evi savi ndwi ndsi bsi msavi2 osavi; do
    # Float results
    stats_float=(${RESULTS_FLOAT[$index]})
    if [ "${stats_float[0]}" != "N/A" ]; then
        validation=$(validate_range "$index" "${stats_float[0]}" "${stats_float[1]}" "${stats_float[2]}")
        scaling_check=$(compare_scaling_results "$index" "${OUTPUT_DIR}/${index}_float.tif" "${OUTPUT_DIR}/${index}_scaled_float.tif")
        
        printf "%-8s %-6s %-8.3f %-8.3f %-8.3f %-8.3f %-15s %-15s\n" \
            "${index^^}" "float" "${stats_float[0]}" "${stats_float[1]}" "${stats_float[2]}" "${stats_float[3]}" \
            "$validation" "$scaling_check"
    fi
    
    # Int16 results  
    stats_int16=(${RESULTS_INT16[$index]})
    if [ "${stats_int16[0]}" != "N/A" ]; then
        # Convert int16 values back to float for validation (divide by scale factor)
        scale_factor=10000
        min_float=$(echo "scale=4; ${stats_int16[0]} / $scale_factor" | bc)
        max_float=$(echo "scale=4; ${stats_int16[1]} / $scale_factor" | bc)
        mean_float=$(echo "scale=4; ${stats_int16[2]} / $scale_factor" | bc)
        
        validation=$(validate_range "$index" "$min_float" "$max_float" "$mean_float")
        
        printf "%-8s %-6s %-8.0f %-8.0f %-8.0f %-8.0f %-15s %-15s\n" \
            "" "int16" "${stats_int16[0]}" "${stats_int16[1]}" "${stats_int16[2]}" "${stats_int16[3]}" \
            "$validation" "N/A"
    fi
    
    # Scaled results
    stats_scaled=(${RESULTS_SCALED_FLOAT[$index]})
    if [ "${stats_scaled[0]}" != "N/A" ]; then
        validation=$(validate_range "$index" "${stats_scaled[0]}" "${stats_scaled[1]}" "${stats_scaled[2]}")
        
        printf "%-8s %-6s %-8.3f %-8.3f %-8.3f %-8.3f %-15s %-15s\n" \
            "" "scaled" "${stats_scaled[0]}" "${stats_scaled[1]}" "${stats_scaled[2]}" "${stats_scaled[3]}" \
            "$validation" "N/A"
    fi
    
    echo ""
done

echo ""
echo "=========================================="
echo "LEGEND"
echo "=========================================="
echo "✅ = Values within expected range"
echo "⚠️  = Values valid but mean outside typical range"  
echo "❌ = Values outside expected theoretical range"
echo ""
echo "VALIDATION WARNINGS:"
echo "- OUT_OF_RANGE: Min/max outside theoretical bounds"
echo "- ATYPICAL_MEAN: Mean outside typical range for land cover"
echo "- CONSTANT_VALUE: All pixels have same value (suspicious)"
echo ""
echo "SCALING CHECK:"
echo "- IDENTICAL: Scaled/unscaled results are the same (expected for ratio indices)"
echo "- DIFFERENT: Results differ (unexpected for ratio indices)"
echo "- EXPECTED_DIFF: Results differ as expected (indices with constants)"
echo ""
echo "EXPECTED RANGES:"
for index in ndvi evi savi ndwi ndsi bsi msavi2 osavi; do
    local expected=(${EXPECTED_RANGES[$index]})
    local typical=(${TYPICAL_RANGES[$index]})
    printf "%-8s: Theoretical [%4.1f, %4.1f], Typical [%4.1f, %4.1f]\n" \
        "${index^^}" "${expected[0]}" "${expected[1]}" "${typical[0]}" "${typical[1]}"
done

echo ""
echo "Test completed!"