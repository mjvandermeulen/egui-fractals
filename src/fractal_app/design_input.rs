use egui::{NumExt as _, Rect, emath::RectTransform};

// TODO!!: change to super::
use crate::{
    FractalApp,
    fractal_app::{
        design_helpers::{closest_handle, closest_line, continue_dragging_line_end},
        tools::max_depth_with_branches,
    },
};

pub fn handle_keyboard_input(ui: &egui::Ui, fractal_app: &mut FractalApp) {
    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];
    // https://github.com/emilk/egui/discussions/1464 -> if. fine tuned with gemini. Maarten.
    if ui.ctx().memory(|mem| mem.focused()).is_none() {
        let max_depth = max_depth_with_branches(
            super::MAX_PAINTED_LINE_COUNT,
            fractal.design_line_count,
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
    let from_screen = to_screen.inverse();

    // let shift_down = ui.input(|i| i.modifiers.shift);
    // if !self.paused
    //     && shift_down
    //     && response.hovered()
    //     && let Some(pointer_pos) = response.hover_pos()

    let id = ui.make_persistent_id("design_painter_interaction");
    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];

    let scroll_response = ui.interact(rect, id, egui::Sense::hover());
    if scroll_response.hovered() {
        ui.input(|input| {
            for event in &input.events {
                if let egui::Event::MouseWheel { delta, .. } = event {
                    // 'delta.y' is the vertical scroll (Mac trackpad two-finger vertical)
                    // 'delta.x' is the horizontal scroll (Mac trackpad two-finger horizontal)
                    fractal.center += from_screen.scale().x * (-1.0 * *delta);
                } else {
                    let zoom_delta = input.zoom_delta();
                    if zoom_delta != 1.0 {
                        fractal.zoom *= zoom_delta;
                    }
                }
            }
        });
    }

    let click_and_drag_response = ui.interact(rect, id, egui::Sense::click_and_drag());

    if click_and_drag_response.is_pointer_button_down_on() {
        // is_pointer_down vs dragged: see tool tip on `dragged`. We don't want a delay.
        if fractal_app.dragged_line_end_point.is_none()
            && let Some(screenpos) = click_and_drag_response.interact_pointer_pos()
        {
            let local_pos = from_screen * screenpos;
            fractal_app.dragged_line_end_point = closest_handle(
                local_pos,
                &fractal.design_lines[..fractal.design_line_count + 1],
                &fractal.lines_style,
            );
        }
        if let Some([line, end]) = fractal_app.dragged_line_end_point {
            continue_dragging_line_end(
                fractal_app,
                from_screen,
                to_screen,
                &click_and_drag_response,
                line,
                end,
            );
        }
    } else {
        fractal_app.dragged_line_end_point = None;
        if click_and_drag_response.double_clicked()
            && let Some(screen_pos) = ui.input(|i| i.pointer.hover_pos())
        {
            let pos = from_screen * screen_pos;
            if let Some(i) = closest_line(pos, &fractal.design_lines, 0.1) {
                let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];

                fractal.design_lines[i].reversed = !fractal.design_lines[i].reversed;
            }
        }
    }
}
