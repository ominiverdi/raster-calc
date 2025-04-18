// benches/benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;
use raster_calc::indices::ndi::calculate_ndi;

fn benchmark_ndi(c: &mut Criterion) {
    c.bench_function("ndi_calculation", |b| {
        b.iter(|| {
            calculate_ndi(
                Path::new("test_data/nir.tif"),
                Path::new("test_data/red.tif"),
                Path::new("test_data/output.tif"),
                true,
                10000
            )
        })
    });
}

criterion_group!(benches, benchmark_ndi);
criterion_main!(benches);