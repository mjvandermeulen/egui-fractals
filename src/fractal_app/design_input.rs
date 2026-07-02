use egui::{NumExt as _, Pos2, Rect, Response, emath::RectTransform};

// TODO!!: change to super::
use crate::{
    FractalApp,
    fractal_app::{
        design_helpers::{
            closest_line, continue_dragging_line_handle, hovered_line_handle, make_loop,
            start_new_line,
        },
        structs_and_enums::{LineHandles, LinesStyle},
        tools::max_depth_with_branches,
    },
};

pub fn handle_keyboard_input(ui: &egui::Ui, fractal_app: &mut FractalApp) {
    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];
    // https://github.com/emilk/egui/discussions/1464 -> if. fine tuned with gemini. Maarten.
    if ui.ctx().memory(|mem| mem.focused()).is_none() {
        // TODO!!!: turn max depth into self.max_depth and calc right away
        // NOPE: only calc max_depth once: right after the design phase
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
                            fractal.depth = (max_depth / 2).at_least(8).at_most(max_depth);
                        } else {
                            fractal.depth = number.at_most(max_depth);
                        }
                    }
                }
            }
        });
        // up and down arrows
        if ui.input_mut(|i| i.key_pressed(egui::Key::ArrowDown)) {
            fractal.depth = fractal.depth.at_least(1) - 1; // avoid subtract with overflow
        }
        if ui.input_mut(|i| i.key_pressed(egui::Key::ArrowUp)) {
            fractal.depth = (fractal.depth + 1).at_most(max_depth);
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
    if fractal_app.trash_line_key_down {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
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
    let id = ui.make_persistent_id("design_painter_interaction");
    let cd_response = ui.interact(rect, id, egui::Sense::click_and_drag());

    // check if dragging continues first
    if let Some((line, handle)) = fractal_app.dragged_handles
        && cd_response.is_pointer_button_down_on()
    {
        continue_dragging_line_handle(
            fractal_app,
            from_screen,
            to_screen,
            &cd_response,
            line,
            &handle,
        );
        return;
    }

    fractal_app.dragged_handles = None;

    let hov_response = ui.interact(rect, id, egui::Sense::hover());
    if !hov_response.hovered() {
        return;
    }
    let Some(global_hover_pos) = hov_response.hover_pos() else {
        return;
    };

    let hover_pos = from_screen * global_hover_pos;

    if start_new_line(ui, fractal_app, &cd_response, hover_pos) {
        return;
    }

    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];
    ui.input(|input| {
        let zoom_delta = input.zoom_delta();
        if zoom_delta != 1.0 {
            fractal.zoom *= zoom_delta;
            return;
        }
        for event in &input.events {
            if let egui::Event::MouseWheel { delta, .. } = event {
                fractal.center += from_screen.scale().x * (-1.0 * *delta);
                return;
            }
        }
    });
    handle_hovered_line_mouse_input(fractal_app, hover_pos, &cd_response);
}

pub fn handle_hovered_line_mouse_input(
    fractal_app: &mut FractalApp,
    local_hover_pos: Pos2,
    click_and_drag_response: &Response,
) {
    // LEARN let ... else
    //   always needs a return
    //   avoid heavy indentation by returning instead of skipping over an indented block
    let Some((hover_line_index, t)) = closest_line(
        local_hover_pos,
        &fractal_app.fractals[fractal_app.fractal_index].design_lines,
        0.1,
    ) else {
        return;
    };

    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];

    fractal_app.hovered_line = Some(hover_line_index);

    if click_and_drag_response.is_pointer_button_down_on() {
        // is_pointer_down vs dragged: see tool tip on `dragged`. We don't want a delay.
        if fractal_app.dragged_handles.is_none()
        // this has to be the case, see above logic
        // && let Some(screenpos) = click_and_drag_response.interact_pointer_pos() CLEAN
        {
            fractal_app.dragged_handles = Some((hover_line_index, hovered_line_handle(t)));
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
