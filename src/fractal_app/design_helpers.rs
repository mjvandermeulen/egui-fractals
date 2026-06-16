use egui::{Color32, Painter, Pos2, Stroke, emath::RectTransform};

use super::structs::VectoredDesignLine;
use super::{DesignLine, LinesStyle};

// pub fn paint_line_handles(
//     painter: &Painter,
//     to_screen: &RectTransform,
//     design_lines: &[DesignLine],
// ) {
//     for design_line in design_lines {
//         for (i, pos) in design_line.line.iter().enumerate() {
//             let center = to_screen * *pos;
//             let r = if i == 0 { 0.02 } else { 0.015 };
//             let radius = r * to_screen.scale().x; // Assuming uniform scaling. AI is clever
//             if i == 0 {
//                 painter.circle_filled(center, radius, Color32::BLACK);
//             } else {
//                 let stroke = Stroke::new(1.0, Color32::BLACK);
//                 painter.circle_stroke(center, radius, stroke);
//             }
//         }
//     }
// }

pub fn closest_handle(
    local_pos: Pos2,
    d_lines: &[DesignLine],
    lines_style: &LinesStyle,
) -> Option<[usize; 2]> {
    let mut min: f32 = 0.05;
    let mut nearest: Option<[usize; 2]> = None;
    for (line_index, design_line) in d_lines.iter().enumerate() {
        for (end_index, end_point) in design_line.line.iter().enumerate() {
            if end_index == 0
                && ((*lines_style == LinesStyle::Tree && line_index != 0)
                    || *lines_style == LinesStyle::Loop)
            {
                continue;
            }
            let d = local_pos.distance(*end_point);
            if d <= min && d < 0.05 {
                min = d;
                nearest = Some([line_index, end_index]);
            }
        }
    }
    nearest
}

pub fn design_lines_to_global_design_vectors(
    local_canvas_lines: &[DesignLine],
    to_screen: RectTransform,
) -> Vec<VectoredDesignLine> {
    local_canvas_lines
        .iter()
        .map(|design_line| VectoredDesignLine::from_design_line(*design_line, to_screen))
        .collect()
}

// total google AI work.
pub fn distance_to_line(p: Pos2, [a, b]: [Pos2; 2]) -> f32 {
    let v = b - a; // Segment vector
    let u = p - a; // Vector to point

    let v_len_sq = v.length_sq();
    if v_len_sq == 0.0 {
        return p.distance(a); // Segment is a single point
    }

    // Project vector u onto v to get parameter t
    let t = u.dot(v) / v_len_sq;

    // Clamp t to keep the closest point on the segment
    let t_clamped = t.clamp(0.0, 1.0);

    // Calculate closest point coordinates
    let closest_point = a + v * t_clamped;

    // Return Euclidean distance
    p.distance(closest_point)
}

pub fn closest_line(local_pos: Pos2, d_lines: &[DesignLine]) -> Option<usize> {
    let mut min: f32 = 0.05;
    let mut nearest: Option<usize> = None;
    for (line_index, design_line) in d_lines.iter().enumerate() {
        let d = distance_to_line(local_pos, design_line.line);
        if d <= min && d < 0.1 {
            min = d;
            nearest = Some(line_index);
        }
    }

    nearest
}

pub fn paint_directed_line_segment(
    painter: &Painter,
    dvec: &VectoredDesignLine,
    width: f32,
    color: Color32,
) {
    let ratio = 0.2;
    let middle = dvec.pos + (1.0 - ratio) * dvec.vec;
    painter.line_segment([dvec.pos, middle], Stroke::new(width, color));
    painter.line_segment(
        [middle, dvec.pos + dvec.vec],
        Stroke::new(width, Color32::BLACK),
    );
}
