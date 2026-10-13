use egui::{Color32, Pos2, Rect, Shape, Vec2};
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
#[must_use]
pub fn line_color(depth: usize, rainbow: bool) -> Color32 {
    if rainbow {
        RAINBOW_COLORS[depth % RAINBOW_COLORS.len()]
    } else {
        Color32::BLACK
    }
}
pub fn paint_line_generator(
    shapes: &mut Vec<Shape>,
    rect: Rect,
    fractal: &Fractal,
) -> impl FnMut(usize, [Pos2; 2], Vec2, Color32) {
    move |depth, line, line_vec, color| {
        let width = if fractal.replace_line {
            if depth != fractal.depth {
                return; // only the final iteration is drawn
            }
            fractal.fixed_final_line_width
        } else {
            line_vec.length() * fractal.initiator_width_length_ratio
        };
        // culling
        if rect.intersects(Rect::from_two_pos(line[0], line[1])) {
            shapes.push(Shape::line_segment(line, (width, color)));
        }
    }
}
fn construct_current_node_and_line(
    parent_node: &Node,
    transform: &LineTransform,
) -> (Node, [Pos2; 2]) {
    let paint_a = parent_node.pos + transform.base_rot * parent_node.vec;
    let paint_vec = transform.rot * parent_node.vec;
    let paint_b = paint_a + paint_vec;
    let painted_node = Node {
        pos: paint_a,
        vec: paint_vec,
    };
    (painted_node, [paint_a, paint_b])
}

// Returns a Vec of Shapes representing the fractal lines painted within the given rect,
// starting AFTER the initiator line, applying the given transformations up to the specified max_depth.
// Aug 30th: This needs to take a closure, since that closure is used in the parallel version and in fractal_app (when bypassing parallelization).
// Oct 9th: bypassing parallelization is still not implemented yet :)
pub fn paint_fractal_lines(
    paint_line: &mut impl FnMut(usize, [egui::Pos2; 2], Vec2, egui::Color32),
    base_node: &Node,
    fractal: &Fractal,
    transformations: &Vec<LineTransform>,
    depth: usize,
    max_depth: usize,
) {
    let mut nodes = vec![Node {
        pos: base_node.pos,
        vec: base_node.vec,
    }];
    let mut new_nodes = Vec::new();

    for d in depth..=max_depth {
        let color = line_color(d, fractal.rainbow);

        if d < max_depth {
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
                let (current_node, line) = construct_current_node_and_line(parent_node, &transform);
                paint_line(1, line, current_node.vec, color);

                // Criterion bench (2 ^ 19 test): wrapping in `if` gives a 8 - 12% improvement.
                if d < max_depth {
                    new_nodes.push(current_node);
                }
            }
        }

        std::mem::swap(&mut nodes, &mut new_nodes);
    }
}

// TODO!!!!!
// CLAUDE:
// A likely bug I spotted:
// In your current parallel_paint_fractal_lines, you call paint_fractal_lines(..., initiator, ..., 2, max_depth). That restarts from the initiator, not from the depth-1 node you just painted, so every top-level branch regenerates the same depth-2+ tree from the root. In the version above I pass node (the depth-1 node) as the start, which fixes that. Taking a Node instead of a &VectoredLine also saves you from converting.
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

            let (current_node, line) = construct_current_node_and_line(&initiator_node, &transform);

            {
                // avoid having to drop the paint_line by indenting. LEARN
                let mut paint_line = paint_line_generator(&mut shapes_iter, rect, fractal);
                paint_line(2, line, current_node.vec, color);

                paint_fractal_lines(
                    &mut paint_line,
                    &current_node,
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
        let mut paint_line = paint_line_generator(&mut shapes, rect, fractal);
        paint_fractal_lines(
            &mut paint_line,
            &Node::from_vectored_line(*initiator), // TODO avoid this deref? Only happens once. Cheap
            fractal,
            transformations,
            1,
            max_depth,
        );
    }
    shapes
}
