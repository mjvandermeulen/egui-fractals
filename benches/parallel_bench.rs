use std::{fs::File, io::BufWriter};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use egui::Rect;
use egui_fractals::structs_and_enums::{Fractal, LineTransform, VectoredLine};
use serde::Serialize;
// use egui_fractals::FractalApp::

fn bench_parallel(c: &mut Criterion) {}

criterion_group!(benches, bench_parallel);
criterion_main!(benches);
