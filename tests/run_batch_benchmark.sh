#!/bin/bash
# tests/run_batch_benchmark.sh
# Script to benchmark batch processing

set -e

# Define paths
BATCH_CONFIG="sentinel_batch_config.json"
OUTPUT_DIR="../data"

# Create output directory if it doesn't exist
mkdir -p $OUTPUT_DIR

echo "Starting comprehensive batch process benchmark..."

# Check if sentinel data exists
SENTINEL_DATA_DIR="../../spectral-calc-tests/data"
if [ ! -f "${SENTINEL_DATA_DIR}/T33TTG_20250305T100029_B08_10m.jp2" ]; then
    echo "Error: Sentinel data not found in ${SENTINEL_DATA_DIR}"
    echo "Please make sure the data is available or update the path in ${BATCH_CONFIG}"
    exit 1
fi

# Check if batch config exists
if [ ! -f "${BATCH_CONFIG}" ]; then
    echo "Error: Batch configuration file not found at ${BATCH_CONFIG}"
    exit 1
fi

# Run the comprehensive batch process
echo "Running batch processing with 16 operations..."

START_TIME=$(date +%s)

# Run batch processing
../target/release/raster-calc batch -c "${BATCH_CONFIG}"

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

echo "=========================================="
echo "Benchmark completed!"
echo "Processing time: ${ELAPSED} seconds"
echo "Average time per operation: $(echo "scale=2; $ELAPSED / 16" | bc) seconds"

# List output files
echo "Generated output files in ${OUTPUT_DIR}:"
ls -lh ${OUTPUT_DIR}/batch_*_*_output.tif | sort || echo "No output files found!"

# Show stats for outputs
echo "Output file statistics:"
for file in $(ls ${OUTPUT_DIR}/batch_*_*_output.tif | sort); do
    if [ -f "$file" ]; then
        filesize=$(du -h "$file" | cut -f1)
        echo "$(basename $file): $filesize"
    fi
done

echo "Benchmark test completed."