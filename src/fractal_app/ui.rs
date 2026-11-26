use egui::{NumExt as _, Pos2, Rect, Response, emath::RectTransform};

use crate::fractal_app::animation::animation_structs_and_enums::AnimationZoomStyle;

use super::{
    FractalApp,
    design_helpers::{
        closest_line, continue_dragging_line_handle, hovered_line_handle, make_loop, start_new_line,
    },
    structs_and_enums::LinesStyle,
    tools::max_depth_with_branches,
};

pub fn handle_keyboard_input(ui: &egui::Ui, fractal_app: &mut FractalApp) {
    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];
    // https://github.com/emilk/egui/discussions/1464. fine tuned with gemini. Maarten.
    if ui.ctx().memory(egui::Memory::focused).is_none() {
        // TODO: turn max depth into self.max_depth and calc right away
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

    // c (select zoom center)
    fractal_app.select_custom_zoom_center = ui.input(|i| i.key_down(egui::Key::C));
    // C cancel zoom center: (consume: so no others can use it and Ctrl-Shift-c won't count)
    if let AnimationZoomStyle::CustomZoomCenter(center) =
        &mut fractal.animation.animation_zoom_style
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::C))
    {
        *center = None;
    }

    // t (trash line)
    fractal_app.trash_line_key_down = ui.input(|i| i.key_down(egui::Key::T));
    if fractal_app.trash_line_key_down {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }

    // l (log a fractal dump)
    if ui.input(|i| i.key_pressed(egui::Key::L)) {
        log::info!("Logging a dump of the current fractal:\n\n {fractal:#?}",);
    }

    // fine tuning (big shout shout out to Claude)
    let (control_pressed, alt_pressed) = ui.input(|i| (i.modifiers.ctrl, i.modifiers.alt));
    fractal_app.fine_tune = match (control_pressed, alt_pressed) {
        (true, true) => 1000.0,
        (true, false) => 100.0,
        (false, true) => 10.0,
        (false, false) => 1.0,
    };
}

pub fn handle_mouse_input(
    ui: &egui::Ui,
    fractal_app: &mut FractalApp,
    to_screen: RectTransform,
    rect: Rect,
) {
    let id = ui.make_persistent_id("design_painter_interaction");
    let from_screen = to_screen.inverse();

    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];

    let click_and_drag_response = ui.interact(rect, id, egui::Sense::click_and_drag());

    if fractal_app.select_custom_zoom_center
        && let AnimationZoomStyle::CustomZoomCenter(center) =
            &mut fractal.animation.animation_zoom_style
    {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);

        if let Some(pos) = ui.input(|i| {
            i.pointer
                .primary_down()
                .then(|| i.pointer.interact_pos())
                .flatten()
        }) {
            *center = Some(pos);
        }
        return;
    }
    // check if dragging continues first
    if let Some((line, handles)) = fractal_app.dragged_handles
        && click_and_drag_response.is_pointer_button_down_on()
    {
        continue_dragging_line_handle(
            fractal_app,
            from_screen,
            to_screen,
            &click_and_drag_response,
            line,
            &handles,
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

    // CLEAN.  This is ugly. AI suggests to return an enum. ? "StartedNewLine", "NothingStarted".
    if fractal_app.new_line_key_down {
        start_new_line(ui, fractal_app, &click_and_drag_response, from_screen);
        return;
    }

    let fractal = &mut fractal_app.fractals[fractal_app.fractal_index];
    ui.input(|input| {
        let zoom_delta = input.zoom_delta();
        if zoom_delta != 1.0 {
            // clippy pedantic complains and suggests an error margin, but that's not needed here. TODO: add an expect
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
    handle_hovered_line_mouse_input(fractal_app, hover_pos, &click_and_drag_response);
}

pub fn handle_hovered_line_mouse_input(
    fractal_app: &mut FractalApp,
    local_hover_pos: Pos2,
    click_and_drag_response: &Response,
) {
    // LEARN let ... else
    //   - always needs a return
    //   - avoids heavy indentation by returning instead of skipping over an indented block
    let Some((hover_line_index, t)) = closest_line(
        local_hover_pos,
        &fractal_app.fractals[fractal_app.fractal_index].design_lines,
        0.1,
        0,
    ) else {
        fractal_app.hovered_line = None;
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
