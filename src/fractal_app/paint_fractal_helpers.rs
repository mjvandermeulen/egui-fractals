use egui::{Color32, Pos2, Rect, Shape, Vec2};

use super::structs_and_enums::{Branch, Fractal, LineTransform};

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
pub fn line_color(depth: usize, color_offset: usize, fractal: &Fractal) -> Color32 {
    if !fractal.rainbow {
        return Color32::BLACK;
    }
    let offset = if fractal.replace_line {
        0
    } else {
        color_offset
    };
    RAINBOW_COLORS[(depth + offset) % RAINBOW_COLORS.len()]
}

pub fn paint_line_generator(
    shapes: &mut Vec<Shape>,
    rect: Rect,
    fractal: &Fractal,
    color: Color32,
    depth: usize,
    max_depth: usize,
) -> impl FnMut([Pos2; 2], Vec2) {
    let final_depth = depth == max_depth;
    move |line, line_vec| {
        let width = if fractal.replace_line {
            if final_depth {
                fractal.fixed_final_line_width
            } else {
                return; // only the final iteration is drawn
            }
        } else {
            line_vec.length() * fractal.initiator_width_length_ratio
        };
        // culling
        if rect.intersects(Rect::from_two_pos(line[0], line[1])) {
            shapes.push(Shape::line_segment(line, (width, color)));
        }
    }
}
pub fn construct_current_branch_and_line(
    parent_branch: &Branch,
    transform: &LineTransform,
) -> (Branch, [Pos2; 2]) {
    let parent_node = parent_branch.parent_node + transform.base_rot * parent_branch.edge;
    let edge = transform.rot * parent_branch.edge;
    let child_node = parent_node + edge;
    let current_branch = Branch { parent_node, edge };
    (current_branch, [parent_node, child_node])
}

// OLD COMMENTS:
// Mutates a Vec of Shapes representing the fractal lines painted within the given rect,
// starting AFTER the initiator line, applying the given transformations up to the specified max_depth.
// Aug 30th: This needs to take a closure, since that closure is used in the parallel version and in fractal_app (when bypassing parallelization).
// Oct 9th: bypassing parallelization is still not implemented yet :)
#[must_use]
pub fn paint_fractal_lines(
    rect: Rect,
    trunk: &Branch,
    fractal: &Fractal,
    transformations: &Vec<LineTransform>,
    max_depth: usize,
    opacity_progress: Option<f32>,
    color_offset: usize,
) -> Vec<Shape> {
    let mut shapes = Vec::new();
    {
        // avoid having to drop the paint_line by indenting. LEARN
        let mut paint_line = paint_line_generator(
            &mut shapes,
            rect,
            fractal,
            line_color(0, color_offset, fractal),
            0,
            max_depth,
        );

        // paint the trunk
        paint_line(
            [trunk.parent_node, trunk.parent_node + trunk.edge],
            trunk.edge,
        );
    }
    let mut branches = vec![*trunk];
    let mut new_branches = Vec::new();

    for d in 1..=max_depth {
        let mut color = line_color(d, color_offset, fractal);
        if d == max_depth
            && let Some(progress) = opacity_progress
            && !fractal.replace_line
        {
            let p = progress.fract();
            color = color.gamma_multiply(p.powf(1.5)); // TODO play with this. Maybe just p * p or just p?
        }
        let mut paint_line = paint_line_generator(&mut shapes, rect, fractal, color, d, max_depth);

        if d < max_depth {
            new_branches.clear();
            new_branches.reserve(branches.len() * transformations.len());
        }

        // iterate over stored parent branches
        //  create a new branch per transformation and paint the line in it
        //  if we're not at the max depth, store the new branch for the next iteration

        // The nesting of the branch loop inside the transformations loop is purely for speed.
        //   It is (just) noticibly faster with a 1 branch depth 17 MIRRORED tree.
        // Benching: at least over 5% slower.
        //   Feel free to read it the other way around, which I think is more intuitive.
        for &transform in transformations {
            for parent_branch in &branches {
                let (current_branch, line) =
                    construct_current_branch_and_line(parent_branch, &transform);
                paint_line(line, current_branch.edge);

                // Criterion bench (2 ^ 19 test): wrapping in `if` gives a 8 - 12% improvement.
                if d < max_depth {
                    new_branches.push(current_branch);
                }
            }
        }

        std::mem::swap(&mut branches, &mut new_branches);
    }
    shapes
}
