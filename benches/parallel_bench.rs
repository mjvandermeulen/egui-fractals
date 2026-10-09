use std::{error::Error, fs::File, io::BufReader};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use egui::Rect;
use egui_fractals::paint_fractal_helpers::parallel_paint_fractal_lines;
use egui_fractals::structs_and_enums::{Fractal, LineTransform, VectoredLine};
use serde::{Deserialize, Serialize};
// use egui_fractals::FractalApp::

fn bench_parallel(c: &mut Criterion) {
    fn read_bench_prep_parallel_struct_from_json()
    -> Result<BenchParallelPaintFractalLinesOwned, Box<dyn Error>> {
        let reader = BufReader::new(File::open("bench_prep_parallel_struct.json")?);
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

    let l = read_bench_prep_parallel_struct_from_json()
        .expect("MAARTEN SAYS: expect bench_parallel json read and parse to succeed.");
    c.bench_function("parallel_paint_fractal", |b| {
        b.iter(|| {
            parallel_paint_fractal_lines(
                black_box(l.rect),
                black_box(&l.initiator),
                black_box(&l.fractal),
                black_box(&l.transformations),
                black_box(l.max_depth),
            )
        });
    });

    //     LEFT OFF HERE:
    //     c.bench_function("regular paint fractal", |b| {
    //         b.iter(

    // paint_fractal_lines(

    //         )
    //     })
}

criterion_group!(benches, bench_parallel);
criterion_main!(benches);
