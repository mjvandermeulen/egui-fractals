use criterion::{Criterion, black_box, criterion_group, criterion_main};
// use egui_fractals::FractalApp::

fn bench_parallel(c: &mut Criterion) {}

criterion_group!(benches, bench_parallel);
criterion_main!(benches);
