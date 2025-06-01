// src/processing/parallel.rs
use crate::utils::cache::RasterCache;
use gdal::Metadata;

use std::{
    collections::HashMap,
    mem,
    num::NonZero,
    ops::DerefMut,
    panic,
    sync::Arc,
    thread::{self, JoinHandle},
};

use anyhow::Result;
use flume::{Receiver, Sender};
use gdal::{
    raster::{Buffer, RasterCreationOptions},
    Dataset, DriverManager, DriverType,
};
use parking_lot::Mutex;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator as _, ParallelIterator as _};

use crate::utils::gdal_ext::TypedBuffer;

type BlockReadHandler = Box<dyn Fn(usize, usize, HashMap<usize, TypedBuffer>) + Send + Sync>;

struct BlockReadRequest {
    datasets: Arc<Vec<Box<[Arc<Mutex<Dataset>>]>>>,
    num_datasets: usize,
    dataset_idx: usize,
    x: usize,
    y: usize,
    state: BlockReadState,
    handler: Arc<BlockReadHandler>,
}

#[derive(Clone)]
struct BlockReadState {
    blocks: Arc<Mutex<HashMap<usize, TypedBuffer>>>,
    region_size: (usize, usize),
}

pub struct ParallelProcessor {
    io_threads: usize,
    cache: Option<Arc<RasterCache>>, // Make cache optional
}

impl ParallelProcessor {
    pub fn new(io_threads: Option<usize>) -> Self {
        let io_threads = io_threads.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(NonZero::get)
                .unwrap_or(4)
                .max(4)
        });

        Self {
            io_threads,
            cache: None,
        }
    }
    pub fn cache_size(&self) -> usize {
        if let Some(cache) = &self.cache {
            cache.len()
        } else {
            0
        }
    }

    pub fn clear_cache(&self) {
        if let Some(cache) = &self.cache {
            cache.clear();
        }
    }

    // Add method to create with cache
    pub fn with_cache(io_threads: Option<usize>, cache: Arc<RasterCache>) -> Self {
        let io_threads = io_threads.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(NonZero::get)
                .unwrap_or(4)
                .max(4)
        });

        Self {
            io_threads,
            cache: Some(cache),
        }
    }

    pub fn process<I: IndexCalculator>(
        &self,
        calculator: I,
        input_paths: &[String],
        output_path: &str,
        use_fixed_point: bool,
        scale_factor: i32,
        input_scale_factor: f32, // NEW PARAMETER
        compress: &str,
        compress_level: u8,
        tiled: bool,
    ) -> Result<()> {
        if input_paths.len() < calculator.required_bands() {
            return Err(anyhow::anyhow!(
                "Not enough input bands provided. Required: {}, provided: {}",
                calculator.required_bands(),
                input_paths.len()
            ));
        }

        // Get input raster dimensions from the first file
        let dataset = Dataset::open(&input_paths[0])?;
        let (width, height) = dataset.raster_size();

        // For small test rasters (like in our tests), use a simple single-threaded approach
        if width <= 512 && height <= 512 {
            return self.process_small_raster(
                calculator,
                input_paths,
                output_path,
                use_fixed_point,
                scale_factor,
                input_scale_factor,
                width,
                height,
                compress,
                compress_level,
                tiled,
            );
        }

        // For larger images, use the parallel block reader
        // Create block reader with cache if available
        let block_reader = if let Some(cache) = &self.cache {
            ParallelBlockReader::with_cache(input_paths, self.io_threads, Arc::clone(cache))
                .map_err(|e| anyhow::anyhow!("Failed to create block reader: {}", e))?
        } else {
            ParallelBlockReader::new(input_paths, self.io_threads)?
        };

        let driver =
            DriverManager::get_output_driver_for_dataset_name(output_path, DriverType::Raster)
                .expect("unknown output format");

        // Create options vector
        let mut options = Vec::new();

        // Add compression if not NONE
        if compress.to_uppercase() != "NONE" {
            options.push(format!("COMPRESS={}", compress.to_uppercase()));

            // Add compression level for supported algorithms
            match compress.to_uppercase().as_str() {
                "DEFLATE" => options.push(format!("ZLEVEL={}", compress_level.min(9))),
                "ZSTD" => options.push(format!("ZSTD_LEVEL={}", compress_level.min(22))),
                _ => {}
            }
        }

        // Add tiling if enabled
        if tiled {
            options.push("TILED=YES".to_string());
        }

        // Always use multi-threading
        options.push("NUM_THREADS=ALL_CPUS".to_string());

        let creation_options = RasterCreationOptions::from_iter(options);

        // Create output dataset with appropriate type
        let mut output = if use_fixed_point {
            driver.create_with_band_type_with_options::<i16, _>(
                output_path,
                width,
                height,
                1,
                &creation_options,
            )?
        } else {
            driver.create_with_band_type_with_options::<f32, _>(
                output_path,
                width,
                height,
                1,
                &creation_options,
            )?
        };

        // Define constants for fixed-point conversion
        const NODATA_VALUE_INT: i16 = -10000;
        const NODATA_VALUE_FLOAT: f32 = -999.0;

        // Set up output dataset properties
        output.set_projection(&dataset.projection())?;
        output.set_geo_transform(&dataset.geo_transform()?.try_into().unwrap())?;

        let mut output_band = output.rasterband(1)?;
        if use_fixed_point {
            output_band.set_no_data_value(Some(NODATA_VALUE_INT as f64))?;
            output_band.set_metadata_item(
                "SCALE",
                &format!("{}", 1.0 / scale_factor as f64),
                "",
            )?;
            output_band.set_metadata_item("OFFSET", "0", "")?;
            output_band.set_description(&format!(
                "{} (scaled by {})",
                calculator.name(),
                scale_factor
            ))?;
        } else {
            output_band.set_no_data_value(Some(NODATA_VALUE_FLOAT as f64))?;
            output_band.set_description(calculator.name())?;
        }

        // Set up processing pipeline
        let (tx, rx) = flume::unbounded();
        let dataset_indices = (0..input_paths.len()).collect::<Vec<_>>();

        // Request processing of each block
        for y in 0..block_reader.blocks.1 {
            for x in 0..block_reader.blocks.0 {
                let tx = tx.clone();
                block_reader.run(
                    x,
                    y,
                    &dataset_indices,
                    Box::new(move |x, y, blocks| {
                        tx.send((x, y, blocks)).unwrap();
                    }),
                );
            }
        }
        drop(tx);

        // Process blocks as they become available
        for (x, y, blocks) in rx {
            // Skip empty blocks (could happen at edges)
            if blocks
                .values()
                .any(|block| block.shape().0 == 0 || block.shape().1 == 0)
            {
                continue;
            }

            // Convert blocks to a vector in the expected order
            let mut inputs = Vec::with_capacity(blocks.len());
            for i in 0..blocks.len() {
                inputs.push(blocks[&i].clone());
            }
            // Apply input scaling if the calculator needs it and scaling factor is not 1.0
            let needs_scaling =
                calculator.needs_input_scaling() && (input_scale_factor - 1.0).abs() > f32::EPSILON;

            println!("🔍 DEBUG: Calculator={}, needs_input_scaling={}, input_scale_factor={}, needs_scaling={}", 
             calculator.name(), calculator.needs_input_scaling(), input_scale_factor, needs_scaling);

             if needs_scaling {
                for buffer in &mut inputs {
                    if let TypedBuffer::F32(ref mut buf) = buffer {
                        println!("BEFORE scaling: First 3 values: {:?}", &buf.data()[0..3.min(buf.data().len())]);
                        for value in buf.data_mut() {
                            *value /= input_scale_factor;
                        }
                        println!("AFTER scaling: First 3 values: {:?}", &buf.data()[0..3.min(buf.data().len())]);
                    }
                }
            }
            // Calculate the index using the provided calculator
            let result = calculator.calculate(&inputs);

            // Get the actual shape of the result
            let result_shape = result.shape();

            // Calculate actual pixel coordinates
            let start_x = x as isize * block_reader.region_size.0 as isize;
            let start_y = y as isize * block_reader.region_size.1 as isize;

            // Skip if we'd be writing out of bounds
            if start_x >= width as isize || start_y >= height as isize {
                continue;
            }

            // Prepare output data
            if use_fixed_point {
                // Convert float result to fixed-point
                let result_data = result.as_f32().unwrap();
                let mut buffer_data = vec![0i16; result_data.data().len()];

                // Apply scaling factor for fixed-point conversion
                for (dst, &src) in buffer_data.iter_mut().zip(result_data.data()) {
                    *dst = if src == NODATA_VALUE_FLOAT {
                        NODATA_VALUE_INT
                    } else {
                        (src.max(-0.9999).min(0.9999) * scale_factor as f32).round() as i16
                    };
                }

                let mut buffer = Buffer::new(result_shape, buffer_data);

                // Write to output
                output_band.write((start_x, start_y), result_shape, &mut buffer)?;
            } else {
                // Use float result directly
                let result_data = result.as_f32().unwrap();

                // Write directly to output
                let mut buffer = Buffer::new(result_shape, result_data.data().to_vec());
                output_band.write((start_x, start_y), result_shape, &mut buffer)?;
            }
        }

        // Finish processing
        block_reader.join();
        Ok(())
    }

    /// Process small rasters (like test images) with a simpler, non-blocked approach
    fn process_small_raster<I: IndexCalculator>(
        &self,
        calculator: I,
        input_paths: &[String],
        output_path: &str,
        use_fixed_point: bool,
        scale_factor: i32,
        input_scale_factor: f32, // NEW PARAMETER
        width: usize,
        height: usize,
        compress: &str,
        compress_level: u8,
        tiled: bool,
    ) -> Result<()> {
        // Define constants for fixed-point conversion
        const NODATA_VALUE_INT: i16 = -10000;
        const NODATA_VALUE_FLOAT: f32 = -999.0;

        // Read all input rasters into memory
        let mut inputs = Vec::with_capacity(input_paths.len());
        for path in input_paths {
            let dataset = Dataset::open(path)?;
            let band = dataset.rasterband(1)?;
            let buffer = band.read_as::<f32>((0, 0), (width, height), (width, height), None)?;
            inputs.push(TypedBuffer::F32(buffer));
        }

        // Apply input scaling if the calculator needs it and scaling factor is not 1.0
        let needs_scaling =
            calculator.needs_input_scaling() && (input_scale_factor - 1.0).abs() > f32::EPSILON;

        if needs_scaling {
            for buffer in &mut inputs {
                if let TypedBuffer::F32(ref mut buf) = buffer {
                    println!(
                        "BEFORE scaling: First 3 values: {:?}",
                        &buf.data()[0..3.min(buf.data().len())]
                    );
                    for value in buf.data_mut() {
                        *value /= input_scale_factor;
                    }
                    println!(
                        "AFTER scaling: First 3 values: {:?}",
                        &buf.data()[0..3.min(buf.data().len())]
                    );
                }
            }
        }
        // Calculate the index
        let result = calculator.calculate(&inputs);

        // Create output dataset
        let driver =
            DriverManager::get_output_driver_for_dataset_name(output_path, DriverType::Raster)
                .expect("unknown output format");

        // Create options vector
        let mut options = Vec::new();

        // Add compression if not NONE
        if compress.to_uppercase() != "NONE" {
            options.push(format!("COMPRESS={}", compress.to_uppercase()));

            // Add compression level for supported algorithms
            match compress.to_uppercase().as_str() {
                "DEFLATE" => options.push(format!("ZLEVEL={}", compress_level.min(9))),
                "ZSTD" => options.push(format!("ZSTD_LEVEL={}", compress_level.min(22))),
                _ => {}
            }
        }

        // Add tiling if enabled
        if tiled {
            options.push("TILED=YES".to_string());
        }

        // Always use multi-threading
        options.push("NUM_THREADS=ALL_CPUS".to_string());

        let creation_options = RasterCreationOptions::from_iter(options);

        let mut output = if use_fixed_point {
            driver.create_with_band_type_with_options::<i16, _>(
                output_path,
                width,
                height,
                1,
                &creation_options,
            )?
        } else {
            driver.create_with_band_type_with_options::<f32, _>(
                output_path,
                width,
                height,
                1,
                &creation_options,
            )?
        };

        // Copy geospatial metadata
        let dataset = Dataset::open(&input_paths[0])?;
        output.set_projection(&dataset.projection())?;
        output.set_geo_transform(&dataset.geo_transform()?.try_into().unwrap())?;

        // Set up band metadata
        let mut output_band = output.rasterband(1)?;
        if use_fixed_point {
            output_band.set_no_data_value(Some(NODATA_VALUE_INT as f64))?;
            output_band.set_metadata_item(
                "SCALE",
                &format!("{}", 1.0 / scale_factor as f64),
                "",
            )?;
            output_band.set_metadata_item("OFFSET", "0", "")?;
            output_band.set_description(&format!(
                "{} (scaled by {})",
                calculator.name(),
                scale_factor
            ))?;

            // Convert and write the result
            let result_data = result.as_f32().unwrap();
            let mut buffer_data = vec![0i16; result_data.data().len()];

            for (dst, &src) in buffer_data.iter_mut().zip(result_data.data()) {
                *dst = if src == NODATA_VALUE_FLOAT {
                    NODATA_VALUE_INT
                } else {
                    (src.max(-0.9999).min(0.9999) * scale_factor as f32).round() as i16
                };
            }

            let mut buffer = Buffer::new(result_data.shape(), buffer_data);
            output_band.write((0, 0), result_data.shape(), &mut buffer)?;
        } else {
            output_band.set_no_data_value(Some(NODATA_VALUE_FLOAT as f64))?;
            output_band.set_description(calculator.name())?;

            // Write the result directly
            let result_data = result.as_f32().unwrap();
            let mut buffer = Buffer::new(result_data.shape(), result_data.data().to_vec());
            output_band.write((0, 0), result_data.shape(), &mut buffer)?;
        }

        Ok(())
    }
}

struct ParallelBlockReader {
    // Instead of storing datasets directly, store paths when using cache
    datasets: Arc<Vec<Box<[Arc<Mutex<Dataset>>]>>>,
    dataset_paths: Option<Arc<Vec<String>>>,
    cache: Option<Arc<RasterCache>>,
    region_size: (usize, usize),
    blocks: (usize, usize),
    workers: Vec<JoinHandle<()>>,
    req_tx: Sender<BlockReadRequest>,
}

impl ParallelBlockReader {
    pub fn new(paths: &[String], threads: usize) -> gdal::errors::Result<Self> {
        let datasets = Arc::new(
            (0..threads)
                .into_par_iter()
                .map(|_| -> gdal::errors::Result<Box<[Arc<Mutex<Dataset>>]>> {
                    Ok(paths
                        .par_iter()
                        .map(|p| -> gdal::errors::Result<Arc<Mutex<Dataset>>> {
                            Ok(Arc::new(Mutex::new(Dataset::open(p)?)))
                        })
                        .collect::<gdal::errors::Result<Vec<_>>>()?
                        .into_boxed_slice())
                })
                .collect::<Result<Vec<_>, _>>()?,
        );

        let (req_tx, req_rx) = flume::unbounded();

        let mut workers = Vec::new();
        for thread_id in 0..threads {
            let req_rx: Receiver<BlockReadRequest> = req_rx.clone();
            let datasets = Arc::clone(&datasets);

            workers.push(thread::spawn(move || {
                for request in req_rx {
                    let block = {
                        let region_size = request.state.region_size;
                        let dataset = datasets[thread_id][request.dataset_idx].lock();
                        let band = dataset.rasterband(1).unwrap();
                        let size = band.size();
                        let window = (request.x * region_size.0, request.y * region_size.1);

                        // Skip if we're completely outside the raster
                        if window.0 >= size.0 || window.1 >= size.1 {
                            TypedBuffer::F32(Buffer::new((0, 0), vec![]))
                        } else {
                            let window_size = (
                                if window.0 + region_size.0 <= size.0 {
                                    region_size.0
                                } else {
                                    size.0 - window.0
                                },
                                if window.1 + region_size.1 <= size.1 {
                                    region_size.1
                                } else {
                                    size.1 - window.1
                                },
                            );

                            let buffer = band
                                .read_as::<f32>(
                                    (window.0 as isize, window.1 as isize),
                                    window_size,
                                    window_size,
                                    None,
                                )
                                .unwrap();

                            TypedBuffer::F32(buffer)
                        }
                    };

                    let blocks = {
                        let mut blocks = request.state.blocks.lock();
                        blocks.insert(request.dataset_idx, block);
                        if blocks.len() == request.num_datasets {
                            let blocks = mem::take(blocks.deref_mut());
                            Some(blocks)
                        } else {
                            None
                        }
                    };

                    if let Some(blocks) = blocks {
                        let BlockReadRequest { handler, .. } = request;
                        (handler)(request.x, request.y, blocks);
                    }
                }
            }));
        }

        let dataset = datasets[0][0].lock();
        let band = dataset.rasterband(1)?;
        let raster_size = band.size();
        let block_size = band.block_size();

        // Use a sensible block size
        let region_size = if block_size.0 > 0 && block_size.1 > 0 {
            // Don't use block sizes larger than the image itself
            (
                block_size.0.min(raster_size.0),
                block_size.1.min(raster_size.1),
            )
        } else {
            // Default size that's never larger than the image
            (256.min(raster_size.0), 256.min(raster_size.1))
        };

        drop(dataset);

        // Calculate number of blocks needed to cover the entire raster
        let blocks = (
            (raster_size.0 + region_size.0 - 1) / region_size.0,
            (raster_size.1 + region_size.1 - 1) / region_size.1,
        );

        Ok(Self {
            datasets,
            dataset_paths: None,
            cache: None,
            region_size,
            blocks,
            workers,
            req_tx,
        })
    }
    // New constructor that uses cache
    pub fn with_cache(
        paths: &[String],
        threads: usize,
        cache: Arc<RasterCache>,
    ) -> anyhow::Result<Self> {
        // Create dataset_paths
        let dataset_paths = Arc::new(paths.to_vec());

        // For dimensions, we temporarily open the first dataset
        let first_dataset = cache.get_dataset(&paths[0])?;
        let band_size = {
            let dataset = first_dataset.lock().unwrap();
            let band = dataset.rasterband(1)?;
            let size = band.size();
            let block_size = band.block_size();
            (size, block_size)
        };

        // Set up the region size and blocks similarly to the original implementation
        let raster_size = band_size.0;
        let block_size = band_size.1;

        let region_size = if block_size.0 > 0 && block_size.1 > 0 {
            (
                block_size.0.min(raster_size.0),
                block_size.1.min(raster_size.1),
            )
        } else {
            (256.min(raster_size.0), 256.min(raster_size.1))
        };

        let blocks = (
            (raster_size.0 + region_size.0 - 1) / region_size.0,
            (raster_size.1 + region_size.1 - 1) / region_size.1,
        );

        // Create request channel
        let (req_tx, req_rx) = flume::unbounded();

        // Set up worker threads
        let mut workers = Vec::with_capacity(threads);
        for _ in 0..threads {
            let req_rx = req_rx.clone();
            let cache = Arc::clone(&cache);
            let dataset_paths = Arc::clone(&dataset_paths);

            let worker = thread::spawn(move || {
                for request in req_rx {
                    let BlockReadRequest {
                        dataset_idx,
                        x,
                        y,
                        state,
                        handler,
                        ..
                    } = request;

                    let path = &dataset_paths[dataset_idx];

                    // Get dataset from cache
                    let result = cache.get_dataset(path);
                    if let Err(e) = result {
                        eprintln!("Error opening dataset {}: {}", path, e);
                        continue;
                    }

                    let dataset_mutex = result.unwrap();

                    // Process block with the dataset
                    let block = {
                        let region_size = state.region_size;
                        let dataset = dataset_mutex.lock().unwrap();
                        let band = dataset.rasterband(1).unwrap();
                        let size = band.size();
                        let window = (x * region_size.0, y * region_size.1);

                        // Same logic as original for reading the block
                        // ...

                        // Read the block
                        if window.0 >= size.0 || window.1 >= size.1 {
                            TypedBuffer::F32(Buffer::new((0, 0), vec![]))
                        } else {
                            let window_size = (
                                if window.0 + region_size.0 <= size.0 {
                                    region_size.0
                                } else {
                                    size.0 - window.0
                                },
                                if window.1 + region_size.1 <= size.1 {
                                    region_size.1
                                } else {
                                    size.1 - window.1
                                },
                            );

                            let buffer = band
                                .read_as::<f32>(
                                    (window.0 as isize, window.1 as isize),
                                    window_size,
                                    window_size,
                                    None,
                                )
                                .unwrap();

                            TypedBuffer::F32(buffer)
                        }
                    };

                    // Use the same logic as original for handling the result
                    let blocks = {
                        let mut blocks = state.blocks.lock();
                        blocks.insert(dataset_idx, block);
                        if blocks.len() == dataset_paths.len() {
                            let blocks = mem::take(blocks.deref_mut());
                            Some(blocks)
                        } else {
                            None
                        }
                    };

                    if let Some(blocks) = blocks {
                        (handler)(x, y, blocks);
                    }
                }
            });

            workers.push(worker);
        }

        Ok(Self {
            datasets: Arc::new(vec![]), // Empty datasets since we're using cache
            dataset_paths: Some(dataset_paths),
            cache: Some(cache),
            region_size,
            blocks,
            workers,
            req_tx,
        })
    }

    // Modify run method to handle both cached and non-cached cases
    pub fn run(
        &self,
        block_x: usize,
        block_y: usize,
        dataset_indices: &[usize],
        handler: BlockReadHandler,
    ) {
        let handler = Arc::new(handler);
        let state = BlockReadState {
            region_size: self.region_size,
            blocks: Arc::new(Mutex::new(HashMap::new())),
        };

        for &idx in dataset_indices {
            let request = BlockReadRequest {
                datasets: self.datasets.clone(),
                num_datasets: dataset_indices.len(),
                dataset_idx: idx,
                x: block_x,
                y: block_y,
                state: state.clone(),
                handler: handler.clone(),
            };
            self.req_tx.send(request).unwrap();
        }
    }

    pub fn join(self) {
        drop(self.req_tx);

        let mut errors = Vec::new();
        for worker in self.workers {
            if let Err(e) = worker.join() {
                errors.push(e);
            }
        }

        if !errors.is_empty() {
            panic::resume_unwind(Box::new(errors));
        }
    }
}

/// Trait for spectral index calculators
pub trait IndexCalculator: Send + Sync {
    /// Calculate the index from the provided input bands
    fn calculate(&self, inputs: &[TypedBuffer]) -> TypedBuffer;

    /// Return the number of required input bands
    fn required_bands(&self) -> usize;

    /// Return the name of the index
    fn name(&self) -> &str;

    /// Return true if this index requires input scaling for proper results
    /// Indices with constants (EVI, SAVI, MSAVI2, OSAVI) need scaling
    /// Pure ratio indices (NDI, NDWI, NDSI, BSI) do not need scaling
    fn needs_input_scaling(&self) -> bool {
        false // Default: most indices are pure ratios that don't need scaling
    }
}
