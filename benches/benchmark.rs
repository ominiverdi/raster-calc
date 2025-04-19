use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gdal::raster::Buffer;
use raster_calc::processing::indices::NDI;
use raster_calc::processing::parallel::IndexCalculator;
use raster_calc::processing::ParallelProcessor;
use raster_calc::utils::gdal_ext::TypedBuffer;
use std::path::Path;

/// Benchmark the core NDI calculation logic in isolation
fn benchmark_ndi_calculation(c: &mut Criterion) {
    // Create synthetic test data
    let size = (1024, 1024);
    let mut band_a_data = vec![0.0f32; size.0 * size.1];
    let mut band_b_data = vec![0.0f32; size.0 * size.1];
    
    // Fill with some test values (simulating NIR and RED bands)
    for i in 0..band_a_data.len() {
        band_a_data[i] = 5000.0 + (i % 100) as f32;
        band_b_data[i] = 2500.0 + (i % 50) as f32;
    }
    
    let band_a = Buffer::new(size, band_a_data);
    let band_b = Buffer::new(size, band_b_data);
    
    let inputs = vec![
        TypedBuffer::F32(band_a),
        TypedBuffer::F32(band_b),
    ];
    
    // Create the NDI calculator
    let ndi = NDI::new(0, 1, None);
    
    // Benchmark the calculation
    c.bench_function("ndi_core_calculation", |b| {
        b.iter(|| ndi.calculate(black_box(&inputs)))
    });
}

/// Benchmark full NDI processing with file I/O
/// Note: This requires test files to exist at the specified paths
fn benchmark_ndi_processing(c: &mut Criterion) {
    // Skip this benchmark if test files don't exist
    let nir_path = "data/nir.tif";
    let red_path = "data/red.tif";
    
    if !Path::new(nir_path).exists() || !Path::new(red_path).exists() {
        println!("Skipping file I/O benchmark - test files not found at {nir_path} and {red_path}");
        return;
    }
    
    // Run once to verify before benchmarking
    {
        let processor = ParallelProcessor::new(None);
        let ndi_instance = NDI::new(0, 1, None);
        let result = processor.process(
            ndi_instance,
            &[nir_path.to_string(), red_path.to_string()],
            "data/benchmark_output.tif",
            true,
            10000,
        );
        
        if let Err(e) = result {
            println!("Skipping file I/O benchmark - test failed: {e}");
            return;
        }
    }
    
    // Now benchmark
    let processor = ParallelProcessor::new(None);
    
    c.bench_function("ndi_file_processing", |b| {
        b.iter(|| {
            // Create a new NDI instance in each iteration
            let ndi_instance = NDI::new(0, 1, None);
            let _ = processor.process(
                black_box(ndi_instance),
                black_box(&[nir_path.to_string(), red_path.to_string()]),
                black_box("data/benchmark_output.tif"),
                black_box(true),
                black_box(10000),
            );
        })
    });
}

// Only benchmark the core calculation for now
criterion_group!(benches, benchmark_ndi_calculation);
criterion_main!(benches);