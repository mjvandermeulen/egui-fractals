#![allow(warnings)]
#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
// TODO!!!! remove this file and all rayon related stuff
// SEE warning suppression above
use egui::{Rect, Shape};
use rayon::prelude::*;

use super::paint_fractal_helpers::{
    construct_current_branch_and_line, line_color, paint_fractal_lines, paint_line_generator,
};
use super::structs_and_enums::{Branch, Fractal, LineTransform, VectoredLine};

#[must_use]
#[inline]
pub fn parallel_paint_fractal_lines(
    rect: Rect,
    initiator: &VectoredLine,
    fractal: &Fractal,
    transformations: &Vec<LineTransform>,
    max_depth: usize,
) -> Vec<Shape> {
    if max_depth < 1 {
        return Vec::new();
    }
    let trunk = Branch {
        parent_node: initiator.pos,
        edge: initiator.vec,
    };
    let color = line_color(1, fractal.rainbow);
    transformations
        .par_iter() // PARALLEL Computation initiated here!
        .with_max_len(1)
        .flat_map(|&transform| {
            let mut shapes_iter = Vec::new();

            let (current_branch, line) = construct_current_branch_and_line(&trunk, &transform);

            {
                // avoid having to drop the paint_line by indenting. LEARN
                let mut paint_line = paint_line_generator(
                    &mut shapes_iter,
                    rect,
                    fractal,
                    line_color(0, fractal.rainbow),
                    0,
                    max_depth,
                );
            }
            paint_fractal_lines(
                &mut shapes_iter,
                rect,
                &current_branch,
                fractal,
                transformations,
                2,
                max_depth,
            );
            shapes_iter
        })
        .collect::<Vec<_>>()
}
