// src/processing/parallel.rs
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
}

impl ParallelProcessor {
    pub fn new(io_threads: Option<usize>) -> Self {
        let io_threads = io_threads.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(NonZero::get)
                .unwrap_or(4)
                .max(4)
        });
        
        Self { io_threads }
    }

    pub fn process<I: IndexCalculator>(
        &self,
        calculator: I,
        input_paths: &[String],
        output_path: &str,
        use_fixed_point: bool,
        scale_factor: i32,
    ) -> Result<()> {
        if input_paths.len() < calculator.required_bands() {
            return Err(anyhow::anyhow!(
                "Not enough input bands provided. Required: {}, provided: {}",
                calculator.required_bands(),
                input_paths.len()
            ));
        }

        let block_reader = ParallelBlockReader::new(input_paths, self.io_threads)?;

        let dataset = Dataset::open(&input_paths[0])?;
        let (width, height) = dataset.raster_size();

        let driver = DriverManager::get_output_driver_for_dataset_name(output_path, DriverType::Raster)
            .expect("unknown output format");
        
        let creation_options =
            RasterCreationOptions::from_iter(["COMPRESS=DEFLATE", "TILED=YES", "NUM_THREADS=ALL_CPUS"]);
        
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
            output_band.set_metadata_item("SCALE", &format!("{}", 1.0 / scale_factor as f64), "")?;
            output_band.set_metadata_item("OFFSET", "0", "")?;
            output_band.set_description(&format!("{} (scaled by {})", calculator.name(), scale_factor))?;
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
        let block_size = block_reader.region_size.0;
        let mut buffer_data = vec![0; block_size * block_size];
        
        for (x, y, blocks) in rx {
            // Convert blocks to a vector in the expected order
            let mut inputs = Vec::with_capacity(blocks.len());
            for i in 0..blocks.len() {
                inputs.push(blocks[&i].clone());
            }
            
            // Calculate the index using the provided calculator
            let result = calculator.calculate(&inputs);
            
            // Prepare output data
            if use_fixed_point {
                // Convert float result to fixed-point
                let result_data = result.as_f32().unwrap();
                buffer_data.resize(result_data.data().len(), 0);
                
                // Apply scaling factor for fixed-point conversion
                for (dst, &src) in buffer_data.iter_mut().zip(result_data.data()) {
                    *dst = if src == NODATA_VALUE_FLOAT {
                        NODATA_VALUE_INT
                    } else {
                        (src.max(-0.9999).min(0.9999) * scale_factor as f32).round() as i16
                    };
                }
                
                let mut buffer = Buffer::new(result_data.shape(), buffer_data);
                output_band.write(
                    (x as isize * block_size as isize, y as isize * block_size as isize),
                    result_data.shape(),
                    &mut buffer,
                )?;
                
                buffer_data = buffer.into_shape_and_vec().1;
            } else {
                // Use float result directly
                let result_data = result.as_f32().unwrap();
                
                // Write directly to output
                let mut buffer = Buffer::new(result_data.shape(), result_data.data().to_vec());
                output_band.write(
                    (x as isize * block_size as isize, y as isize * block_size as isize),
                    result_data.shape(),
                    &mut buffer,
                )?;
            }
        }

        // Finish processing
        block_reader.join();
        Ok(())
    }
}

struct ParallelBlockReader {
    datasets: Arc<Vec<Box<[Arc<Mutex<Dataset>>]>>>,
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
        // Fall back to a reasonable block size if not specified by the driver
        let region_size = if block_size.0 > 0 && block_size.1 > 0 {
            block_size
        } else {
            (1024, 1024) // Default to a reasonable size
        };
        drop(dataset);

        let blocks = (
            (raster_size.0 + region_size.0 - 1) / region_size.0,
            (raster_size.1 + region_size.1 - 1) / region_size.1,
        );

        Ok(Self {
            datasets,
            region_size,
            blocks,
            workers,
            req_tx,
        })
    }

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
}