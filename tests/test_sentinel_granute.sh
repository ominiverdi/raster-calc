# NDVI (float32)
time ../target/release/raster-calc ndi \
  -a ../../spectral-calc-tests/data/T33TTG_20250305T100029_B08_10m.jp2   \
   -b ../../spectral-calc-tests/data/T33TTG_20250305T100029_B04_10m.jp2  \
     -o ../data/ndvi_large_float.tif --float

# NDVI (int16)
time ../target/release/raster-calc ndi \
  -a ../../spectral-calc-tests/data/T33TTG_20250305T100029_B08_10m.jp2   \
   -b ../../spectral-calc-tests/data/T33TTG_20250305T100029_B04_10m.jp2  \
     -o ../data/ndvi_large_int16.tif 