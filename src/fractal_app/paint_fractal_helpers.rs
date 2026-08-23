use egui::{Color32, Pos2, Rect, Shape};

use super::structs_and_enums::{Fractal, LineTransform, Node, VectoredLine};

// Closure Factory:
// returns a closure that paints lines into the shapes Vec, but only if they intersect the given rect.
pub fn paint_line_generator(
    shapes: &mut Vec<Shape>,
    rect: Rect,
) -> impl FnMut([Pos2; 2], Color32, f32) {
    move |points: [Pos2; 2], color: Color32, width: f32| {
        let line: [Pos2; 2] = [points[0], points[1]];
        // culling
        if rect.intersects(Rect::from_two_pos(line[0], line[1])) {
            shapes.push(Shape::line_segment(line, (width, color)));
        }
    }
}

// Adds to the shapes Vec the lines of the fractal, up to the specified depth.
// NOTE: It takes the paint_line closure to add the lines to the the captured (by the closure) shapes Vec.
pub fn paint_fractal_lines(
    paint_line: &mut dyn FnMut([egui::Pos2; 2], Color32, f32),
    initiator: &VectoredLine,
    fractal: &Fractal,
    transformations: &Vec<LineTransform>,
    max_depth: usize,
) {
    let mut nodes = vec![Node {
        pos: initiator.pos,
        vec: initiator.vec,
    }];

    let mut new_nodes = Vec::new();
    for depth in 1..=max_depth {
        let color = line_color(depth, fractal.rainbow);

        if depth < max_depth {
            new_nodes.clear();
            new_nodes.reserve(nodes.len() * 2);
        }

        // iterate over stored parent nodes
        //  create a new node per transformation and paint the line in it
        //  if we're not at the max depth, store the new node for the next iteration

        // the nesting of the node loop inside the transformations loop is purely for speed
        //   it is (just) noticibly faster with a 1 branch depth 17 mirrorred tree
        // Feel free to read it the other way around.
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
