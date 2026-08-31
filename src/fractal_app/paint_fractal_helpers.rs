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
pub fn paint_line_generator(
    shapes: &mut Vec<Shape>,
    rect: Rect,
) -> impl FnMut([egui::Pos2; 2], egui::Color32, f32) {
    move |points: [Pos2; 2], color: Color32, width: f32| {
        let line: [Pos2; 2] = [points[0], points[1]];
        // culling
        if rect.intersects(Rect::from_two_pos(line[0], line[1])) {
            shapes.push(Shape::line_segment(line, (width, color)));
        }
    }
}

// Paints a single line into the shapes Vec, but only if it intersects the given rect.
// This is a convenience function that uses the paint_line_generator closure factory.
// It is a little convoluted, but since it's used rarely there is not performance issue, and it keeps things DRY.
// TODO!!: replace this function with it's own one line body everywhere in the code.
// Does that drop the paint_line? YES, tested
pub fn paint_one_line(
    shapes: &mut Vec<Shape>,
    rect: Rect,
    points: [Pos2; 2],
    color: Color32,
    width: f32,
) {
    paint_line_generator(shapes, rect)(points, color, width);
}

// Returns a Vec of Shapes representing the fractal lines painted within the given rect,
// starting AFTER the initiator line, applying the given transformations up to the specified max_depth.
// Aug 30th: This needs to take a closure, since that closure is used in the parallel version and in fractal_app (when bypassing parallelization).
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
            new_nodes.reserve(nodes.len() * 2);
        }

        // iterate over stored parent nodes
        //  create a new node per transformation and paint the line in it
        //  if we're not at the max depth, store the new node for the next iteration

        // the nesting of the node loop inside the transformations loop is purely for speed
        //   it is (just) noticibly faster with a 1 branch depth 17 MIRRORED tree
        // Feel free to read it the other way around, which I think is more intuitive.
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
                if depth < max_depth {
                    new_nodes.push(painted_node);
                }
            }
        }

        std::mem::swap(&mut nodes, &mut new_nodes);
    }
}

#[must_use]
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
        .par_iter()
        .with_max_len(1)
        .flat_map(|&transform| {
            let mut shapes_iter = Vec::new();

            let paint_a = initiator_node.pos + transform.base_rot * initiator_node.vec;
            let paint_vec = transform.rot * initiator_node.vec;
            let paint_b = paint_a + paint_vec;

            {
                // avoid having to drop the paint_line, by indenting LEARN
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
            // Call paint_fractal_lines for each transformation parallelly.
            // Change paint_fractal_lines to take a closure again....
            // paint_fractal_lines(rect, initiator, fractal, transformations, max_depth)
            // LEFT OFF HERE: Call paint_fractal_lines with new depth!
            shapes_iter
        })
        .collect::<Vec<_>>()
}
