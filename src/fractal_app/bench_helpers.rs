use std::fs::File;
use std::io::BufWriter;

use egui::Rect;
use serde::Serialize;

use crate::fractal_app::structs_and_enums::{Fractal, LineTransform, VectoredLine};

pub fn save_bench_prep_parallel_struct_to_json(
    rect: Rect,
    initiator: &VectoredLine,
    fractal: &Fractal,
    transformations: &Vec<LineTransform>,
    max_depth: usize,
) {
    #[derive(Debug, Serialize)]
    pub struct BenchParallelPaintFractalLinesRef<'a> {
        pub rect: Rect,
        pub initiator: &'a VectoredLine,
        pub fractal: &'a Fractal,
        pub transformations: &'a Vec<LineTransform>,
        pub max_depth: usize,
    }

    let bench = BenchParallelPaintFractalLinesRef {
        rect,
        initiator,
        fractal,
        transformations,
        max_depth,
    };

    let file =
        File::create("bench_prep_parallel_struct.json").expect("Expect no prob with file creation");
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &bench).expect("Expect no prob with writing to JSON file");
}
