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
use itertools::izip;
use parking_lot::Mutex;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator as _, ParallelIterator as _};

use gdal_ext::TypedBuffer;

mod gdal_ext;

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
        for _thread_id in 0..threads {
            let req_rx: Receiver<BlockReadRequest> = req_rx.clone();

            workers.push(thread::spawn(move || {
                for request in req_rx {
                    let block = {
                        let region_size = request.state.region_size;
                        let dataset = request.datasets[_thread_id][request.dataset_idx].lock();
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

                        // println!(
                        //     "Reading block {}, {} in dataset {} on thread {}",
                        //     request.x, request.y, request.idx, thread_id
                        // );

                        let buffer = band
                            .read_as::<u16>(
                                (window.0 as isize, window.1 as isize),
                                window_size,
                                window_size,
                                None,
                            )
                            .unwrap();

                        TypedBuffer::U16(buffer)
                        // band.read_typed_block(request.x, request.y).unwrap()
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
        // let block_size = (2048, 2048);
        let _geo_transform = dataset.geo_transform()?;
        drop(dataset);

        let region_size = block_size;
        let blocks = (
            raster_size.0.div_ceil(block_size.0),
            raster_size.1.div_ceil(block_size.1),
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

pub fn main() -> Result<()> {
    let granule_path = "../data/";
    // let nir_path = format!("{}T33TTG_20250305T100029_B8A_20m.jp2", granule_path);
    // let red_path = format!("{}T33TTG_20250305T100029_B04_20m.jp2", granule_path);
    let nir_path = format!("{}T33TTG_20250305T100029_B08_10m.jp2", granule_path);
    let red_path = format!("{}T33TTG_20250305T100029_B04_10m.jp2", granule_path);
    let output_path = "../output/rust_parallel_io.tif";

    let inputs = [nir_path, red_path];
    let _io_threads = 4.max(
        std::thread::available_parallelism()
            .unwrap_or(NonZero::<usize>::MIN)
            .get(),
    );
    let io_threads = 8;

    let block_reader = ParallelBlockReader::new(&inputs, io_threads)?;

    let dataset = Dataset::open(&inputs[0])?;
    let (width, height) = dataset.raster_size();

    let driver = DriverManager::get_output_driver_for_dataset_name(output_path, DriverType::Raster)
        .expect("unknown output format");
    let creation_options =
        RasterCreationOptions::from_iter(["COMPRESS=DEFLATE", "TILED=YES", "NUM_THREADS=ALL_CPUS"]);
    let mut output = driver.create_with_band_type_with_options::<i16, _>(
        output_path,
        width,
        height,
        1,
        &creation_options,
    )?;

    const SCALING_FACTOR: f32 = 10000.0;
    const NODATA_VALUE: i16 = -20000;

    output.set_projection(&dataset.projection())?;
    output.set_geo_transform(&dataset.geo_transform()?)?;
    output
        .rasterband(1)?
        .set_no_data_value(Some(NODATA_VALUE as f64))?;

    let (tx, rx) = flume::unbounded();
    let dataset_indices = (0..inputs.len()).collect::<Vec<_>>();
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

    let block_size = 1024; // FIXME
    let mut ndvi_data = vec![0; block_size * block_size];
    for (x, y, mut blocks) in rx {
        let nir_block = blocks.remove(&0).unwrap();
        let red_block = blocks.remove(&1).unwrap();
        let nir_block = nir_block.as_u16().unwrap();
        let red_block = red_block.as_u16().unwrap();

        ndvi_data.resize(nir_block.data().len(), 0);
        for (ndvi, nir, red) in izip!(
            ndvi_data.iter_mut(),
            nir_block.data().iter().copied(),
            red_block.data().iter().copied()
        ) {
            let red = red as f32 - 1000.0;
            let nir = nir as f32 - 1000.0;

            let t = if red + nir > 0.0 {
                ((nir - red) / (nir + red)).clamp(-1.0, 1.0)
            } else {
                NODATA_VALUE as f32
            };

            *ndvi = (t * SCALING_FACTOR).round_ties_even() as i16;
        }

        let mut buffer = Buffer::new(nir_block.shape(), ndvi_data);

        let mut output_band = output.rasterband(1)?;

        output_band.write(
            (
                x as isize * block_size as isize,
                y as isize * block_size as isize,
            ),
            red_block.shape(),
            &mut buffer,
        )?;

        ndvi_data = buffer.into_shape_and_vec().1;
    }

    block_reader.join();

    Ok(())
}
