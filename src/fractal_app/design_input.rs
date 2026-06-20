use egui::NumExt as _;

use crate::{FractalApp, fractal_app::tools::max_depth_with_branches};

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
