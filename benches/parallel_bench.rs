use std::{error::Error, fs::File, io::BufReader};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use egui::Rect;
use egui_fractals::paint_fractal_helpers::sequential_paint_fractal_lines;

use egui_fractals::parallel_paint_fractal::parallel_paint_fractal_lines;
use egui_fractals::structs_and_enums::{Fractal, LineTransform, VectoredLine};
use serde::{Deserialize, Serialize};
// use egui_fractals::FractalApp::

fn bench_parallel(c: &mut Criterion) {
    fn read_bench_prep_parallel_struct_from_json(
        path: &'static str,
    ) -> Result<BenchParallelPaintFractalLinesOwned, Box<dyn Error>> {
        let reader = BufReader::new(File::open(path)?);
        let loaded: BenchParallelPaintFractalLinesOwned = serde_json::from_reader(reader)?;
        Ok(loaded)
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct BenchParallelPaintFractalLinesOwned {
        rect: Rect,
        initiator: VectoredLine,
        fractal: Fractal,
        transformations: Vec<LineTransform>,
        max_depth: usize,
    }

    let mut loaded_binary_fractal =
        read_bench_prep_parallel_struct_from_json("bench_prep_bin_tree_parallel_struct.json")
            .expect("MAARTEN SAYS: expect bench_parallel json read and parse to succeed.");

    // #########################################################################
    // NOTE: increase max_depth for testing
    // #########################################################################

    loaded_binary_fractal.max_depth += 2;

    c.bench_function("sequential_paint_fractal_bin_tree", |b| {
        b.iter(|| {
            sequential_paint_fractal_lines(
                black_box(loaded_binary_fractal.rect),
                black_box(&loaded_binary_fractal.initiator),
                black_box(&loaded_binary_fractal.fractal),
                black_box(&loaded_binary_fractal.transformations),
                black_box(loaded_binary_fractal.max_depth),
            )
        });
    });

    c.bench_function("parallel_paint_fractal_bin_tree", |b| {
        b.iter(|| {
            parallel_paint_fractal_lines(
                black_box(loaded_binary_fractal.rect),
                black_box(&loaded_binary_fractal.initiator),
                black_box(&loaded_binary_fractal.fractal),
                black_box(&loaded_binary_fractal.transformations),
                black_box(loaded_binary_fractal.max_depth),
            )
        });
    });

    let mut loaded_multi_branch_fractal =
        read_bench_prep_parallel_struct_from_json("bench_prep_11_branch_parallel_struct.json")
            .expect("MAARTEN SAYS: expect bench_parallel json read and parse to succeed.");

    // #########################################################################
    // NOTE: increase max_depth for testing
    // #########################################################################

    loaded_multi_branch_fractal.max_depth += 2;

    c.bench_function("sequential_paint_fractal_b_11", |b| {
        b.iter(|| {
            sequential_paint_fractal_lines(
                black_box(loaded_multi_branch_fractal.rect),
                black_box(&loaded_multi_branch_fractal.initiator),
                black_box(&loaded_multi_branch_fractal.fractal),
                black_box(&loaded_multi_branch_fractal.transformations),
                black_box(loaded_multi_branch_fractal.max_depth),
            )
        });
    });

    c.bench_function("parallel_paint_fractal_b_11", |b| {
        b.iter(|| {
            parallel_paint_fractal_lines(
                black_box(loaded_multi_branch_fractal.rect),
                black_box(&loaded_multi_branch_fractal.initiator),
                black_box(&loaded_multi_branch_fractal.fractal),
                black_box(&loaded_multi_branch_fractal.transformations),
                black_box(loaded_multi_branch_fractal.max_depth),
            )
        });
    });
}

criterion_group!(benches, bench_parallel);
criterion_main!(benches);
