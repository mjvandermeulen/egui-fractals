use egui::{NumExt as _, Rect, emath::RectTransform};

// TODO!!: change to super::
use crate::{
    FractalApp,
    fractal_app::{
        design_helpers::{
            closest_handle, closest_line, closest_line_handle, continue_dragging_line_end,
            draw_new_line, make_loop,
        },
        structs::LinesStyle,
        tools::max_depth_with_branches,
    },
};

pub fn handle_keyboard_input(ui: &egui::Ui, fractal_app: &mut FractalApp) {
    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];
    // https://github.com/emilk/egui/discussions/1464 -> if. fine tuned with gemini. Maarten.
    if ui.ctx().memory(|mem| mem.focused()).is_none() {
        // TODO: turn max depth into self.max_depth and calc right away
        let max_depth = max_depth_with_branches(
            super::MAX_PAINTED_LINE_COUNT,
            fractal.design_lines.len() - 1,
            fractal.mirror,
            fractal.replace_line,
        );
        // read number keys
        ui.ctx().input(|i| {
            for event in &i.events {
                if let egui::Event::Text(text) = event {
                    // Check if the typed character is a digit
                    if text.chars().any(|c| c.is_ascii_digit())
                        && let Ok(number) = text.parse::<usize>()
                    {
                        if number == 9 {
                            fractal.depth = max_depth;
                        } else if number == 8 {
                            // NOTE: max_depth could be < 8, so you can't clamp(8, max_depth);
                            fractal.depth = (max_depth / 2).at_least(8).clamp(0, max_depth);
                        } else {
                            fractal.depth = number.clamp(0, max_depth);
                        }
                    }
                }
            }
        });
        // up and down arrows
        if fractal.depth > 0 //clamping doesn't avoid a usize overflow soon enough
                && ui.input_mut(|i| i.key_pressed(egui::Key::ArrowDown))
        {
            fractal.depth = (fractal.depth - 1).clamp(0, max_depth);
        }
        if ui.input_mut(|i| i.key_pressed(egui::Key::ArrowUp)) {
            fractal.depth = (fractal.depth + 1).clamp(0, max_depth);
        }
    }

    // d (design)

    if ui.input(|i| i.modifiers.shift_only()) {
        // shift down
        if ui.input(|i| i.key_pressed(egui::Key::D)) {
            fractal_app.show_design_only = !fractal_app.show_design_only;
        }
    } else {
        // shift up
        if ui.input(|i| i.key_down(egui::Key::D)) {
            fractal_app.show_design_only = true;
        } else if ui.input(|i| i.key_released(egui::Key::D)) {
            fractal_app.show_design_only = false;
        }
    }

    // n (new line)
    fractal_app.new_line_key_down = ui.input(|i| i.key_down(egui::Key::N));

    // t (trash line)
    fractal_app.trash_line_key_down = ui.input(|i| i.key_down(egui::Key::T));

    // l (log a fractal dump)
    if ui.input(|i| i.key_down(egui::Key::L)) {
        log::info!("Log a dump of the current fractal: {fractal:#?}",);
    }

    fractal_app.fine_tune = ui.input(|i| i.modifiers.ctrl);
}

pub fn handle_mouse_input(
    ui: &egui::Ui,
    fractal_app: &mut FractalApp,
    to_screen: RectTransform,
    rect: Rect,
) {
    if fractal_app.trash_line_key_down {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }

    let from_screen = to_screen.inverse();
    let id = ui.make_persistent_id("design_painter_interaction");
    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];
    let click_and_drag_response = ui.interact(rect, id, egui::Sense::click_and_drag());

    // check if dragging continues first
    if let Some([line, end]) = fractal_app.dragged_line_end_point
        && click_and_drag_response.is_pointer_button_down_on()
    {
        continue_dragging_line_end(
            fractal_app,
            from_screen,
            to_screen,
            &click_and_drag_response,
            line,
            end,
        );
        return;
    }

    fractal_app.dragged_line_end_point = None;

    let hover_response = ui.interact(rect, id, egui::Sense::hover());
    if hover_response.hovered()
        && let Some(global_hover_pos) = hover_response.hover_pos()
    {
        let hover_pos = from_screen * global_hover_pos;
        fractal_app.hovered_line = closest_line(hover_pos, &fractal.design_lines, 0.1);
        if draw_new_line(ui, fractal_app, &click_and_drag_response, hover_pos) {
            return;
        }

        let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];
        ui.input(|input| {
            let zoom_delta = input.zoom_delta();
            if zoom_delta != 1.0 {
                fractal.zoom *= zoom_delta;
                // TODO: turn off fractal_app.hovered_line OR turn off the return!!!!!
                return;
            }
            for event in &input.events {
                if let egui::Event::MouseWheel { delta, .. } = event {
                    // 'delta.y' is the vertical scroll (Mac trackpad two-finger vertical)
                    // 'delta.x' is the horizontal scroll (Mac trackpad two-finger horizontal)
                    fractal.center += from_screen.scale().x * (-1.0 * *delta);
                    // TODO: turn off fractal_app.hovered_line
                    return;
                }
            }
        });
        if let Some(hover_line_index) = fractal_app.hovered_line {
            fractal_app.hovered_line = Some(hover_line_index);

            if click_and_drag_response.is_pointer_button_down_on() {
                // is_pointer_down vs dragged: see tool tip on `dragged`. We don't want a delay.
                if fractal_app.dragged_line_end_point.is_none() // this has to be the case, see above logic
                && let Some(screenpos) = click_and_drag_response.interact_pointer_pos()
                {
                    let local_pos = from_screen * screenpos; // NOTE: this should be the same as hover_pos...
                    if let Some((handle, _)) = closest_line_handle(
                        local_pos,
                        &fractal.design_lines[hover_line_index],
                        f32::MAX,
                        false,
                    ) {
                        fractal_app.dragged_line_end_point = Some([hover_line_index, handle]);
                    }
                }
            } else if click_and_drag_response.double_clicked() {
                if fractal_app.trash_line_key_down {
                    if hover_line_index != 0 {
                        fractal.design_lines.remove(hover_line_index);
                        if fractal.lines_style == LinesStyle::Loop {
                            make_loop(fractal_app);
                        }
                    } else {
                        log::info!("can't remove the base line (index == 0)");
                    }
                } else {
                    let design_line = &mut fractal.design_lines[hover_line_index];
                    design_line.reversed = !design_line.reversed;
                }
            }
        }
    }
}
