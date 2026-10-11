use egui::{Color32, Pos2, Rect, Shape};
use rayon::prelude::*;

use super::structs_and_enums::{Fractal, LineTransform, Node, VectoredLine};

const RAINBOW_COLORS: [Color32; 6] = [
    Color32::from_rgb(255, 0, 0),   // Red
    Color32::from_rgb(255, 127, 0), // Orange
    Color32::from_rgb(255, 255, 0), // Yellow
    Color32::from_rgb(0, 255, 0),   // Green
    Color32::from_rgb(0, 0, 255),   // Blue
    Color32::from_rgb(139, 0, 255), // Magenta (a more visually distinct purple)
];
// const OLD_RAINBOW_COLORS: [Color32; 6] = [Color32::RED,Color32::ORANGE,Color32::YELLOW,Color32::GREEN,Color32::BLUE,Color32::MAGENTA];

pub fn line_color(depth: usize, rainbow: bool) -> Color32 {
    if rainbow {
        RAINBOW_COLORS[depth % RAINBOW_COLORS.len()]
    } else {
        Color32::BLACK
    }
}

// Closure Factory:
// returns a closure that paints lines into the shapes Vec, but only if they intersect the given rect.
// LEARN: Claude: "You typically need move when the closure outlives the scope it was created in."
pub fn paint_line_generator(
    shapes: &mut Vec<Shape>,
    rect: Rect,
) -> impl FnMut([egui::Pos2; 2], egui::Color32, f32) {
    move |points: [Pos2; 2], color: Color32, width: f32| {
        // culling
        if rect.intersects(Rect::from_two_pos(points[0], points[1])) {
            shapes.push(Shape::line_segment(points, (width, color)));
        }
    }
}

// Returns a Vec of Shapes representing the fractal lines painted within the given rect,
// starting AFTER the initiator line, applying the given transformations up to the specified max_depth.
// Aug 30th: This needs to take a closure, since that closure is used in the parallel version and in fractal_app (when bypassing parallelization).
// Oct 9th: bypassing parallelization is still not implemented yet :)
pub fn paint_fractal_lines(
    paint_line: &mut impl FnMut([egui::Pos2; 2], egui::Color32, f32),
    initiator: &VectoredLine,
    fractal: &Fractal,
    transformations: &Vec<LineTransform>,
    depth: usize,
    max_depth: usize,
) {
    let mut nodes = vec![Node {
        pos: initiator.pos,
        vec: initiator.vec,
    }];
    let mut new_nodes = Vec::new();

    for depth in depth..=max_depth {
        let color = line_color(depth, fractal.rainbow);

        if depth < max_depth {
            new_nodes.clear();
            // new_nodes.reserve(nodes.len() * 2); // TODO!!!!!! whoa, update and bench this!!! SEE NEXT LINE
            new_nodes.reserve(nodes.len() * transformations.len());
        }

        // iterate over stored parent nodes
        //  create a new node per transformation and paint the line in it
        //  if we're not at the max depth, store the new node for the next iteration

        // The nesting of the node loop inside the transformations loop is purely for speed.
        //   It is (just) noticibly faster with a 1 branch depth 17 MIRRORED tree.
        // Benching: at least over 5% slower.
        //   Feel free to read it the other way around, which I think is more intuitive.
        for &transform in transformations {
            for parent_node in &nodes {
                let paint_a = parent_node.pos + transform.base_rot * parent_node.vec;
                let paint_vec = transform.rot * parent_node.vec;
                let paint_b = paint_a + paint_vec;
                let painted_node = Node {
                    pos: paint_a,
                    vec: paint_vec,
                };

                if fractal.replace_line {
                    if depth == max_depth {
                        paint_line([paint_a, paint_b], color, fractal.fixed_final_line_width);
                    }
                    // else: do not paint the line, just store the new node for the next iteration
                } else {
                    paint_line(
                        [paint_a, paint_b],
                        color,
                        painted_node.vec.length() * fractal.initiator_width_length_ratio,
                    );
                }
                // Criterion bench (2 ^ 19 test): wrapping in `if` gives a 8 - 12% improvement.
                if depth < max_depth {
                    new_nodes.push(painted_node);
                }
            }
        }

        std::mem::swap(&mut nodes, &mut new_nodes);
    }
}

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
    let initiator_node = Node {
        pos: initiator.pos,
        vec: initiator.vec,
    };
    let color = line_color(1, fractal.rainbow);

    transformations
        .par_iter() // PARALLEL Computation initiated here!
        .with_max_len(1)
        .flat_map(|&transform| {
            let mut shapes_iter = Vec::new();

            let paint_a = initiator_node.pos + transform.base_rot * initiator_node.vec;
            let paint_vec = transform.rot * initiator_node.vec;
            let paint_b = paint_a + paint_vec;

            {
                // avoid having to drop the paint_line by indenting. LEARN
                let mut paint_line = paint_line_generator(&mut shapes_iter, rect);
                if fractal.replace_line {
                    if max_depth == 1 {
                        paint_line([paint_a, paint_b], color, fractal.fixed_final_line_width);
                    }
                    // else: do not paint the line
                } else {
                    paint_line(
                        [paint_a, paint_b],
                        color,
                        paint_vec.length() * fractal.initiator_width_length_ratio,
                    );
                }
                paint_fractal_lines(
                    &mut paint_line,
                    initiator,
                    fractal,
                    transformations,
                    2,
                    max_depth,
                );
            }
            shapes_iter
        })
        .collect::<Vec<_>>()
}

// for benching
// maybe needed for WASM?
#[must_use]
#[inline]
pub fn sequential_paint_fractal_lines(
    rect: Rect,
    initiator: &VectoredLine,
    fractal: &Fractal,
    transformations: &Vec<LineTransform>,
    max_depth: usize,
) -> Vec<Shape> {
    let mut shapes = Vec::new();
    {
        // avoid having to drop the paint_line by indenting. LEARN
        let mut paint_line = paint_line_generator(&mut shapes, rect);
        paint_fractal_lines(
            &mut paint_line,
            initiator,
            fractal,
            transformations,
            1,
            max_depth,
        );
    }
    shapes
}
