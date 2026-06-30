use super::structs_and_enums::VectoredDesignLine;
use super::{DesignLine, LinesStyle};
use crate::FractalApp;
use crate::fractal_app::structs_and_enums::LineHandle;
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

pub fn closest_line_handle(
    local_pos: Pos2,
    dl: &DesignLine,
    threshold: f32,
) -> Option<(usize, f32)> {
    let mut min = threshold;
    let mut result: Option<(usize, f32)> = None;
    for (end_index, end_point) in dl.line.iter().enumerate() {
        let d = local_pos.distance(*end_point);
        if d <= min {
            min = d;
            result = Some((end_index, min));
        }
    }
    result
}
pub fn hovered_line_handle(t: f32) -> LineHandle {
    match t {
        0.0..=0.25 => LineHandle::SingleHandle(0),
        0.25..=0.75 => LineHandle::DoubleHandle,
        _ => LineHandle::SingleHandle(1),
    }
}

pub fn closest_handle(
    local_pos: Pos2,
    dlines: &[DesignLine],
    threshold: f32,
) -> Option<[usize; 2]> {
    let mut min = threshold;
    let mut nearest_handle: Option<[usize; 2]> = None;
    for (i, dl) in dlines.iter().enumerate() {
        if let Some((closest, dist)) = closest_line_handle(local_pos, dl, min) {
            min = dist;
            nearest_handle = Some([i, closest]);
        }
    }
    nearest_handle
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
// calculates the distance to a line
// returns:
//   - distance
//   - ratio of where on the line the closest point on the segment is between a and b
//       e.g.: a: 0.0, b: 1.0  middle: 0.5
fn distance_to_line(p: Pos2, [a, b]: [Pos2; 2]) -> (f32, f32) {
    let v = b - a; // Segment vector
    let u = p - a; // Vector to point

    let v_len_sq = v.length_sq();
    if v_len_sq == 0.0 {
        return (p.distance(a), 0.5); // Segment is a single point
    }

    // Project vector u onto v to get parameter t
    let t = u.dot(v) / v_len_sq;

    // Clamp t to keep the closest point on the segment
    let t_clamped = t.clamp(0.0, 1.0);

    // Calculate closest point coordinates
    let closest_point = a + v * t_clamped;

    // Return Euclidean distance and t
    (p.distance(closest_point), t)
}

pub fn closest_line(
    local_pos: Pos2,
    design_lines: &[DesignLine],
    threshold: f32,
) -> Option<(usize, f32)> {
    let mut min: f32 = threshold;
    let mut nearest: Option<(usize, f32)> = None;
    for (line_index, design_line) in design_lines.iter().enumerate() {
        let (d, t) = distance_to_line(local_pos, design_line.line);
        if d <= min {
            min = d;
            nearest = Some((line_index, t));
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

pub fn continue_dragging_line_handle(
    fractal_app: &mut FractalApp,
    from_screen: RectTransform,
    to_screen: RectTransform,
    click_and_drag_response: &Response,
    line: usize,
    handle: &LineHandle,
) {
    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];

    let tuning_ratio = if fractal_app.fine_tune { 0.02 } else { 1.0 };
    let target_handles: Vec<(usize, usize, Pos2)> = match handle {
        LineHandle::SingleHandle(end_index) => {
            vec![(
                line,
                *end_index,
                from_screen
                    * (to_screen * fractal.design_lines[line].line[*end_index]
                        + tuning_ratio * click_and_drag_response.drag_delta()),
            )]
        }
        LineHandle::DoubleHandle => {
            let mut handles = vec![];
            for end_index in 0..=1 {
                handles.push((
                    line,
                    end_index,
                    from_screen
                        * (to_screen * fractal.design_lines[line].line[end_index]
                            + tuning_ratio * click_and_drag_response.drag_delta()),
                ));
            }
            handles
        }
    };
    log::info!("target handles: {target_handles:#?}");
    for (mut line_index, mut handle_index, new_pos) in target_handles {
        match fractal.lines_style {
            LinesStyle::Loop => {
                if handle_index == 0 {
                    // move the previous line tip (index == 1)
                    line_index = (line_index + fractal.design_lines.len() - 1)
                        % (fractal.design_lines.len());
                }
                fractal.design_lines[line_index].line[1] = new_pos;
                let next_line_index = (line_index + 1) % (fractal.design_lines.len());
                fractal.design_lines[next_line_index].line[0] = new_pos;
            }
            LinesStyle::Tree => {
                // BUG: tree does not stay connected
                // NOTE: closest handle used to have this code:
                //       if end_index == 0
                //     && ((*lines_style == LinesStyle::Tree && line_index != 0)
                //         || *lines_style == LinesStyle::Loop)
                // {
                //     continue;
                if line_index != 0 && handle_index == 0 {
                    line_index = 0;
                    handle_index = 1;
                }
                fractal.design_lines[line_index].line[handle_index] = new_pos;
                if line_index == 0 && handle_index == 1 {
                    fractal
                        .design_lines
                        .iter_mut()
                        .skip(1)
                        .for_each(|d_line| d_line.line[0] = new_pos);
                }
            }
            _ => {
                log::info!("moving line {line_index} handle {handle_index}");
                fractal.design_lines[line_index].line[handle_index] = new_pos;
            }
        }
    }
}

pub fn start_new_line(
    // draw new line depending on LineStyle TODO!!!!!
    ui: &egui::Ui,
    fractal_app: &mut FractalApp,
    cd_response: &Response,
    hover_pos: Pos2,
) -> bool {
    if !fractal_app.new_line_key_down {
        return false;
    }
    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    if cd_response.is_pointer_button_down_on() {
        let new_line = DesignLine {
            line: [hover_pos, hover_pos], // TODO change depending on tree or loop
            reversed: false,
        };
        let design_lines = &mut fractal_app.fractals[fractal_app.fractal_index].design_lines;
        let new_line_index = design_lines.len();
        design_lines.push(new_line);
        fractal_app.dragged_line = Some((new_line_index, LineHandle::SingleHandle(1)));
    }
    true
}

pub fn make_loop(fractal_app: &mut FractalApp) {
    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];
    let base = fractal.design_lines[0];
    let mut remaining_lines = fractal.design_lines.split_off(1);
    let mut new_dls: Vec<DesignLine> = Vec::with_capacity(remaining_lines.len());

    let mut current_pos = base.line[1];
    while !remaining_lines.is_empty() {
        match closest_handle(current_pos, &remaining_lines, f32::MAX) {
            None => {
                // TODO!! replace all over: closest => nearest
                log::warn!("A closest line should always be found here");
                break;
            }
            Some([index, handle]) => {
                let mut dl = remaining_lines.remove(index);
                if handle == 1 {
                    dl.line.swap(0, 1);
                    dl.reversed = !dl.reversed;
                }
                dl.line[0] = current_pos;
                current_pos = dl.line[1];
                new_dls.push(dl);
            }
        }
    }
    let new_len = new_dls.len();
    if new_len > 1 && new_dls[new_len - 1].line[0] == base.line[0] {
        // the last line would disappear
        new_dls[new_len - 2].line[1] = new_dls[new_len - 1].line[1];
        new_dls[new_len - 1].line[0] = new_dls[new_len - 1].line[1];
    }
    if new_len > 0 {
        new_dls[new_len - 1].line[1] = base.line[0]; // close the loop
    }
    fractal.design_lines.extend(new_dls);
}

pub fn handle_line_style_change(fractal_app: &mut FractalApp) {
    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];
    match fractal.lines_style {
        LinesStyle::Free => {}
        LinesStyle::Tree => {
            let (base, not_base_lines) = fractal.design_lines.split_at_mut(1);
            let base_tip = base[0].line[1];
            for dl in not_base_lines {
                match closest_line_handle(base_tip, dl, f32::MAX) {
                    None => log::warn!("A closest line should always be found here"),
                    Some((handle, _)) => {
                        if handle == 1 {
                            dl.line.swap(0, 1);
                            dl.reversed = !dl.reversed;
                        }
                        dl.line[0] = base_tip;
                    }
                }
            }
        }
        LinesStyle::Loop => {
            make_loop(fractal_app);
        }
    }
}
