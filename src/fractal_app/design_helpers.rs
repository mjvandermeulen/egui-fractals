use super::structs::VectoredDesignLine;
use super::{DesignLine, LinesStyle};
use crate::FractalApp;
use egui::Response;
use egui::{Color32, Painter, Pos2, Stroke, emath::RectTransform};

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

pub fn closest_line(local_pos: Pos2, design_lines: &[DesignLine], threshold: f32) -> Option<usize> {
    let mut min: f32 = threshold;
    let mut nearest: Option<usize> = None;
    for (line_index, design_line) in design_lines.iter().enumerate() {
        let d = distance_to_line(local_pos, design_line.line);
        if d <= min {
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

pub fn continue_dragging_line_end(
    fractal_app: &mut FractalApp,
    from_screen: RectTransform,
    to_screen: RectTransform,
    click_and_drag_response: &Response,
    line: usize,
    end: usize,
) {
    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];

    let tuning_ratio = if fractal_app.fine_tune { 0.02 } else { 1.0 };
    let new_point = from_screen
        * (to_screen * fractal.design_lines[line].line[end]
            + tuning_ratio * click_and_drag_response.drag_delta());
    if fractal.lines_style == LinesStyle::Loop {
        debug_assert_ne!(
            end, 0,
            "Loop style expects that the start point of a line can not be dragged"
        );
        fractal.design_lines[line].line[1] = new_point;
        let next_line_index = (line + 1) % (fractal.design_lines.len());
        fractal.design_lines[next_line_index].line[0] = new_point;
    } else if fractal.lines_style == LinesStyle::Tree {
        fractal.design_lines[line].line[end] = new_point;
        if line == 0 && end == 1 {
            fractal
                .design_lines
                .iter_mut()
                .skip(1)
                .for_each(|d_line| d_line.line[0] = new_point);
        }
    } else {
        fractal.design_lines[line].line[end] = new_point;
    }
}

pub fn draw_new_line(
    ui: &egui::Ui,
    fractal_app: &mut FractalApp,
    cd_response: &Response,
    hover_pos: Pos2,
) -> bool {
    if !fractal_app.new_line_key_down {
        fractal_app.new_line = None;
        return false;
    }

    match fractal_app.new_line.as_mut() {
        None => {
            // First mouse button down after new_line_key_down
            if cd_response.is_pointer_button_down_on() {
                fractal_app.new_line = Some(DesignLine {
                    line: [hover_pos, hover_pos],
                    reversed: false,
                });
            }
        }
        Some(nl) => {
            if ui.input(|i| i.pointer.primary_pressed()) {
                // Second click pressed this frame

                log::info!("New DesignLine: {nl:#?}");
                fractal_app.new_line = None;
            } else if cd_response.is_pointer_button_down_on() {
                // button not release yet after first mouse button down after new_line_key_down
                fractal_app.new_line = Some(DesignLine {
                    line: [hover_pos, hover_pos],
                    reversed: false,
                });
                // CLEANUP: SAME AS ABOVE...
                //   FIRST HANDLE THE ` Second click pressed this frame`
            } else {
                nl.line[1] = hover_pos;
            }
        }
    }
    true
}
