use egui::{Color32, Pos2, Rect, Shape};
use rayon::prelude::*;

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
// TODO!!!!! PONDER: THIS CAN TAKE A CLOSURE AGAIN... given by parallel_paint_fractal_lines
#[must_use]
pub fn paint_fractal_lines(
    rect: Rect,
    initiator: &VectoredLine,
    fractal: &Fractal,
    transformations: &Vec<LineTransform>,
    max_depth: usize,
) -> Vec<Shape> {
    let mut nodes = vec![Node {
        pos: initiator.pos,
        vec: initiator.vec,
    }];
    let mut new_nodes = Vec::new();
    let mut shapes: Vec<Shape> = Vec::new();

    {
        // indendation needed to drop the closure before returning shapes, otherwise we get a borrow error
        //   this way we avoid:
        //   - we avoid refactoring out the part where the closure is used.
        //   - having to manually drop the closure.
        let mut paint_line = paint_line_generator(&mut shapes, rect);

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
    shapes
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

// // TODO!!!!!: ????? can take the same closure that painted the initiator... ???????????????
// #[must_use]
// pub fn parallel_paint_fractal_lines(
//     rect: Rect,
//     initiator: &VectoredLine,
//     fractal: &Fractal,
//     transformations: &Vec<LineTransform>,
//     max_depth: usize,
// ) -> Vec<Shape> {
//     let mut nodes = vec![Node {
//         pos: initiator.pos,
//         vec: initiator.vec,
//     }];
//     let mut new_nodes = Vec::new();

//     {
//         // indendation needed to drop the closure before returning shapes, otherwise we get a borrow error
//         //   this way we avoid:
//         //   - we avoid refactoring out the part where the closure is used.
//         //   - having to manually drop the closure. (hmmm, now I think that would be cleaner)
//         for depth in 1..=max_depth {
//             let color = line_color(depth, fractal.rainbow);

//             if depth < max_depth {
//                 new_nodes.clear();
//                 new_nodes.reserve(nodes.len() * 2);
//             }

//             // iterate over stored parent nodes
//             //  create a new node per transformation and paint the line in it
//             //  if we're not at the max depth, store the new node for the next iteration

//             // woah LEFT OFF HERE. I'm kinda stumped. the pseudo recursion is biting me.
//             for parent_node in &nodes {
//                 transformations
//                     .par_iter()
//                     .with_max_len(1)
//                     .flat_map(|transform| {
//                         let mut paint_line = paint_line_generator(&mut shapes, rect);
//                         // NOTE THIS closures needs to be passed on to paint_fractal_lines!!!!!!!!!!!

//                         let paint_a = parent_node.pos + transform.base_rot * parent_node.vec;
//                         let paint_vec = transform.rot * parent_node.vec;
//                         let paint_b = paint_a + paint_vec;
//                         let painted_node = Node {
//                             pos: paint_a,
//                             vec: paint_vec,
//                         };

//                         if fractal.replace_line {
//                             if depth == max_depth {
//                                 paint_line(
//                                     [paint_a, paint_b],
//                                     color,
//                                     fractal.fixed_final_line_width,
//                                 );
//                             }
//                             // else: do not paint the line, just store the new node for the next iteration
//                         } else {
//                             paint_line(
//                                 [paint_a, paint_b],
//                                 color,
//                                 painted_node.vec.length() * fractal.initiator_width_length_ratio,
//                             );
//                         }
//                         if depth < max_depth {
//                             // FIRST: REMOVE the parent node loop:
//                             //  - only one depth
//                             //  - only one initiator node
//                             new_nodes.push(painted_node); // LEFT OFF HERE: instead of pushing the new node, we need to call paint_fractal_lines with this new "initiator" and the same transformations, but with max_depth - depth (=1)
//                         }
//                     })
//                     .collect::<Vec<_>>();
//             }

//             std::mem::swap(&mut nodes, &mut new_nodes);
//         }
//     }
// }
