#![allow(warnings)] // TODO
use std::{error::Error, fs::File, io::BufReader};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use egui::Rect;

use egui_fractals::{
    paint_fractal_helpers::paint_fractal_lines,
    structs_and_enums::{Fractal, LineTransform, VectoredLine},
};
use serde::{Deserialize, Serialize};

// USAGE:
// cargo bench
// cargo bench "seq.*tree" # only run benches that match regex "seq.*tree"
// TODO: better grouping. Ask Claude.

#[derive(Debug, Serialize, Deserialize)]
struct BenchParallelPaintFractalLinesOwned {
    rect: Rect,
    initiator: VectoredLine,
    fractal: Fractal,
    transformations: Vec<LineTransform>,
    max_depth: usize,
}

fn read_bench_prep_parallel_struct_from_json(
    path: &'static str,
) -> Result<BenchParallelPaintFractalLinesOwned, Box<dyn Error>> {
    let reader = BufReader::new(File::open(path)?);
    let loaded: BenchParallelPaintFractalLinesOwned = serde_json::from_reader(reader)?;
    Ok(loaded)
}

fn bench_paint(c: &mut Criterion) {
    let mut loaded_binary_fractal =
        read_bench_prep_parallel_struct_from_json("bench_prep_bin_tree_parallel_struct.json")
            .expect("MAARTEN SAYS: expect bench_parallel json read and parse to succeed.");

    // #########################################################################
    // NOTE: max_depth increased for testing
    // #########################################################################

    loaded_binary_fractal.max_depth += 2;

    // c.bench_function("paint_fractal_lines_bin_tree", |b| {
    //     b.iter(|| {
    //         paint_fractal_lines(
    //             black_box(loaded_binary_fractal.rect),
    //             black_box(&loaded_binary_fractal.initiator),
    //             black_box(&loaded_binary_fractal.fractal),
    //             black_box(&loaded_binary_fractal.transformations),
    //             black_box(1),
    //             black_box(loaded_binary_fractal.max_depth),
    //             black_box(None),
    //         )
    //     });
    // });

    let mut loaded_multi_branch_fractal =
        read_bench_prep_parallel_struct_from_json("bench_prep_11_branch_parallel_struct.json")
            .expect("MAARTEN SAYS: expect bench_parallel json read and parse to succeed.");

    // #########################################################################
    // NOTE: max_depth increased for testing
    // #########################################################################

    loaded_multi_branch_fractal.max_depth += 2;

    // Hackish Claude idea to limit sample of this slow function
    // let mut slow = c.benchmark_group("parallel_slow");
    // slow.sample_size(20);
    // slow.bench_function("sequential_paint_fractal_b_11", |b| {
    //     b.iter(|| {
    //         sequential_paint_fractal_lines(
    //             black_box(loaded_multi_branch_fractal.rect),
    //             black_box(&loaded_multi_branch_fractal.initiator),
    //             black_box(&loaded_multi_branch_fractal.fractal),
    //             black_box(&loaded_multi_branch_fractal.transformations),
    //             black_box(1),
    //             black_box(loaded_multi_branch_fractal.max_depth),
    //             black_box(None),
    //         )
    //     });
    // });
}

criterion_group!(benches, bench_paint);
criterion_main!(benches);
