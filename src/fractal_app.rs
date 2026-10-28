mod animation;
mod bench_helpers;
mod design_helpers;
mod design_input;
mod fractals;
pub mod paint_fractal_helpers; // pub for benching check if needed
pub mod structs_and_enums; // pub for benching TODO check if needed
mod tools;

use std::time::Instant;

use egui::{
    Button, Color32, NumExt as _, Painter, Pos2, Rect, Stroke, Ui,
    containers::{CollapsingHeader, Frame},
    emath::RectTransform,
    pos2,
    widgets::Slider,
};

use animation::{
    animation_tools::find_animation_rotation_center,
    scale_and_rotate_vectored_lines_around_rotation_center,
};
use bench_helpers::save_bench_prep_parallel_struct_to_json;
use design_helpers::{
    handle_line_style_change, paint_directed_line_segment, reversible_lines_to_global_line_vectors,
};
use design_input::{handle_keyboard_input, handle_mouse_input};
use fractals::fractals;
use paint_fractal_helpers::paint_fractal_lines;
use structs_and_enums::{
    Branch, Fractal, LineHandles, LineTransform, LinesStyle, ReversibleLine, VectoredLine,
};
use tools::max_depth_with_branches;

use crate::{
    fractal_app::animation::design_to_frame_scale_rotation_and_color_offset,
    structs_and_enums::AnimationState,
};

// TODO: find the error that pops op when the vec gets too large.
const MAX_PAINTED_LINE_COUNT: usize = (1 << 18) + 100; // 2 to the power of 18 + 1. HARDCODED

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FractalApp {
    fractals: Vec<Fractal>,
    fractal_index: usize,
    #[serde(skip)]
    line_count: usize,
    #[serde(skip)]
    fine_tune: f32, // normal 1.0, ten times (Alt) 10.0 and 100 times (Ctrl) 100.0
    #[serde(skip)]
    dragged_handles: Option<(usize, LineHandles)>, // Add option for incorrect drag. Now it catches an endpoint when dragging over it, after starting in the middle of nowhere :)
    #[serde(skip)]
    show_design_only: bool,
    #[serde(skip)]
    new_line_key_down: bool,
    #[serde(skip)]
    trash_line_key_down: bool,
    #[serde(skip)]
    hovered_line: Option<usize>, // for coloring the hovered line neon green.
    // NOTE: when dragging over a non green line, it will "pick up" the line
    // TODO: include LineHandles in the hovered_line.

    // TODO! put all animation stuff in a struct.
    #[serde(skip)]
    animation_state: AnimationState,
    // TODO! progress: f32, // for animation. instead of local, so you can pause and resume.
    #[serde(skip)]
    animation_cycle: usize, // one cycle is when a generator line is animated to take exactly the place of the initiator.
    #[serde(skip)]
    new_cycle: bool,
    #[serde(skip)]
    animation_cycle_start_limited_depth_paint_count: usize,
    #[serde(skip)]
    a_b_c: Option<(Pos2, Pos2, Pos2)>, // for animation. HACK TEMP
    #[serde(skip)]
    bench_prep: bool,
}

impl Default for FractalApp {
    fn default() -> Self {
        Self {
            fractals: fractals(),
            fractal_index: 0,
            line_count: 0,
            show_design_only: false,
            fine_tune: 1.0,
            dragged_handles: None,
            new_line_key_down: false,
            trash_line_key_down: false,
            hovered_line: None,
            animation_state: AnimationState::Stopped,
            animation_cycle: 0,
            new_cycle: true, // works, but semi ugly.
            animation_cycle_start_limited_depth_paint_count: 0,
            a_b_c: None,
            bench_prep: false,
        }
    }
}

impl FractalApp {
    /// Called once before the first frame.
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::light());

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    #[expect(clippy::too_many_lines)]
    fn options_ui(&mut self, ui: &mut Ui) {
        egui::ComboBox::from_label("Select starter Fractal")
            .selected_text(&self.fractals[self.fractal_index].name)
            .show_ui(ui, |ui| {
                for (index, fractal) in self.fractals.iter().enumerate() {
                    ui.selectable_value(&mut self.fractal_index, index, &fractal.name);
                }
            });

        let max_depth;
        {
            let fractal = &mut self.fractals[self.fractal_index];
            if ui
                .add_enabled(
                    *fractal != Self::default().fractals[self.fractal_index],
                    Button::new(format!("Reset {}", fractal.name)),
                )
                .on_hover_text(format!("Reset only the {} fractal", fractal.name))
                .clicked()
            {
                *fractal = Self::default().fractals[self.fractal_index].clone();
            }
            if ui
                .add_enabled(
                    // TODO clippy pedantic warns about comparing floats:
                    // (fractal.zoom - Self::default
                    //  ().fractals[self.fractal_index].zoom).abs() > error_margin
                    // same for .center
                    fractal.zoom != Self::default().fractals[self.fractal_index].zoom
                        || fractal.center != Self::default().fractals[self.fractal_index].center,
                    Button::new("Reset View"),
                )
                .on_hover_text("Reset the zoom and scroll")
                .clicked()
            {
                fractal.zoom = Self::default().fractals[self.fractal_index].zoom;
                fractal.center = Self::default().fractals[self.fractal_index].center;
            }

            max_depth = max_depth_with_branches(
                MAX_PAINTED_LINE_COUNT,
                fractal.design_lines.len() - 1,
                fractal.mirror,
                fractal.replace_line,
            );

            ui.label(format!("Painted line count: {}", self.line_count));
            ui.checkbox(&mut fractal.replace_line, "Replace parent with children");
            ui.checkbox(&mut fractal.mirror, "Mirror");
            ui.checkbox(&mut fractal.rainbow, "Rainbow");
            ui.radio_value(&mut fractal.lines_style, LinesStyle::Free, "Free");
            ui.radio_value(&mut fractal.lines_style, LinesStyle::Tree, "Tree");
            ui.radio_value(&mut fractal.lines_style, LinesStyle::Loop, "Loop");
            ui.add(Slider::new(&mut fractal.zoom, 0.001..=5.0).text("zoom"));
            if fractal.replace_line {
                ui.add(
                    Slider::new(&mut fractal.fixed_final_line_width, 0.05..=1.1)
                        .logarithmic(true)
                        .text("Final line width"),
                );
            } else {
                ui.add(
                    egui::Slider::new(&mut fractal.initiator_width_length_ratio, 0.001..=0.07)
                        .custom_formatter(|x, _| format!("{:.1}%", x * 100.0))
                        .text("width % of line length"),
                );
            }
            ui.add(Slider::new(&mut fractal.depth, 0..=max_depth).text("depth"));
        }

        egui::reset_button(ui, self, "Full Reset"); // NOTE: will not look disabled, because of self.line_count

        // ---------------------------------------------------------------------
        ui.separator();
        let fractal = &mut self.fractals[self.fractal_index];

        use egui::{Button, RichText};
        use std::time::{Duration, Instant};

        ui.horizontal(|ui| {
            let is_playing = matches!(self.animation_state, AnimationState::Playing { .. });
            let is_stopped = matches!(self.animation_state, AnimationState::Stopped);

            // Play: resumes from the paused time, or starts from ZERO after a stop
            if ui
                .add_enabled(!is_playing, Button::new(RichText::new("▶").size(18.0)))
                .on_hover_text("Play (resume the animation)")
                .clicked()
            {
                let previously_elapsed_time = match self.animation_state {
                    AnimationState::Paused { elapsed } => elapsed,
                    _ => Duration::ZERO,
                };
                self.animation_state = AnimationState::Playing {
                    previously_elapsed_time,
                    start_time: Instant::now(),
                };
            }

            // Pause: stores the total time played so far
            if ui
                .add_enabled(is_playing, Button::new(RichText::new("⏸").size(18.0)))
                .on_hover_text("Pause")
                .clicked()
            {
                if let AnimationState::Playing {
                    previously_elapsed_time,
                    start_time,
                } = self.animation_state
                {
                    self.animation_state = AnimationState::Paused {
                        elapsed: previously_elapsed_time + start_time.elapsed(),
                    };
                }
            }

            // Stop: resets the animation
            if ui
                .add_enabled(!is_stopped, Button::new(RichText::new("⏹").size(18.0)))
                .on_hover_text("Stop (reset the animation)")
                .clicked()
            {
                self.animation_state = AnimationState::Stopped;
            }
        });
        ui.add(
            Slider::new(&mut fractal.animation.animation_repetition_cycle, 0..=10)
                .text("cycle that can be repeated"),
        );

        // if let Some(animation) = &mut fractal.animation { TODO
        // ui.add(
        //     Slider::new(&mut fractal.animation.length, 0.5..=30.0)
        //         .text("Animation length (seconds)"),
        // );

        // ---------------------------------------------------------------------
        ui.separator();

        if ui
            .add_enabled(!self.bench_prep, egui::Button::new("Prep for Benchmark"))
            .clicked()
        {
            self.bench_prep = true;
        }

        // ---------------------------------------------------------------------
        ui.separator();

        ui.add(egui::github_link_file!(
            "https://github.com/mjvandermeulen/egui-fractals/blob/main/",
            "Source code."
        ));
        ui.hyperlink_to(
            "README.md (with Instructions)",
            "https://github.com/mjvandermeulen/egui-fractals/blob/main/",
        );
    }

    fn design(&mut self, ui: &Ui, to_screen: RectTransform, painter: &Painter) {
        handle_keyboard_input(ui, self);
        handle_mouse_input(ui, self, to_screen, painter.clip_rect());

        let fractal = &mut self.fractals[self.fractal_index];
        fractal.depth = fractal.depth.at_most(max_depth_with_branches(
            MAX_PAINTED_LINE_COUNT,
            fractal.design_lines.len() - 1,
            fractal.mirror,
            fractal.replace_line,
        ));
    }
    // TODO: if not painting abc, then no self needed and this can move to own helper library
    fn animation_frame_vectors(
        &mut self,
        gdvs: Vec<VectoredLine>, // global design vectors
        progress: f32,
        repetition_cycle: usize,
    ) -> (Vec<VectoredLine>, usize) {
        let cycle_angle = gdvs[0].vec.angle() - gdvs[1].vec.angle();
        let cycle_scale = gdvs[0].vec.length() / gdvs[1].vec.length();

        // TODO: better naming
        // HACK, check if there is a generator line. TODO!!!!!

        let b = gdvs[1].pos;
        let c = gdvs[0].pos;
        let rotation_center = find_animation_rotation_center(b, c, cycle_angle, cycle_scale);
        // To VISUALIZE the a b c triangle:
        self.a_b_c = Some((rotation_center.unwrap_or(Pos2::new(-2.0, -1.0)), b, c));

        let (scale, rotation, color_offset) = design_to_frame_scale_rotation_and_color_offset(
            cycle_angle,
            cycle_scale,
            progress,
            repetition_cycle,
        );
        // HACK, check if there is a generator line. TODO!!!!!

        (
            scale_and_rotate_vectored_lines_around_rotation_center(
                gdvs,
                rotation_center.unwrap_or(Pos2::new(-2.0, -1.0)), // UGLY HACK, TODO
                scale,
                rotation,
            ),
            color_offset,
        )
    }

    fn paint_design(&self, painter: &Painter, design_vectors: &[VectoredLine]) {
        let width_length_ratio = self.fractals[self.fractal_index].initiator_width_length_ratio;
        let highlight_color =
            Color32::from_hex("#0FFF50").expect("Expected hex neon green to be parsed correctly");
        design_vectors.iter().enumerate().for_each(|(i, vec)| {
            // LEARN `if Some(i) == self.hovered_line` below. This is sooo nice
            let color = if Some(i) == self.hovered_line {
                highlight_color
            } else if i == 0 {
                Color32::RED
            } else {
                Color32::ORANGE
            };
            paint_directed_line_segment(painter, vec, width_length_ratio, color);
        });
    }

    // TODO remove
    fn paint_fractal(&mut self, painter: &Painter, vectored_design_lines: &[VectoredLine]) {
        let fractal = &self.fractals[self.fractal_index];
        let debug_assert_max_depth = max_depth_with_branches(
            MAX_PAINTED_LINE_COUNT,
            vectored_design_lines.len() - 1,
            fractal.mirror,
            fractal.replace_line,
        );
        debug_assert!(
            fractal.depth <= debug_assert_max_depth,
            "fractal.depth = {}, max_depth_with_branches(...) = {debug_assert_max_depth}",
            fractal.depth
        );
        let rect = painter.clip_rect();

        let initiator = vectored_design_lines[0];
        let transformations: Vec<LineTransform> = vectored_design_lines[1..]
            .iter()
            .flat_map(|line| {
                let mut line_transforms: Vec<LineTransform> =
                    vec![LineTransform::from_design_vector(&initiator, *line, false)];
                if fractal.mirror {
                    line_transforms
                        .push(LineTransform::from_design_vector(&initiator, *line, true));
                }
                line_transforms
            })
            .collect();

        if self.bench_prep {
            self.bench_prep = false;
            save_bench_prep_parallel_struct_to_json(
                rect,
                &initiator,
                fractal,
                &transformations,
                fractal.depth,
            )
            .expect("Expected save_bench_prep_parallel_struct_to_json not to cause any trouble");
        }

        let shapes = paint_fractal_lines(
            rect,
            &Branch::from_vectored_line(initiator),
            fractal,
            &transformations,
            fractal.depth,
            None,
            0,
        );
        self.line_count = shapes.len();
        // TODO!!!!: Bench and try to avoid `let shapes`, just push to painter...
        // ~~never mind: the std::mem::swap trick won't work.~~ HA HA: That's for the branches. It will work!!!!
        painter.extend(shapes);
    }

    // TODO rename to paint_fractal
    fn animate(&mut self, painter: &Painter, global_design_vectors: Vec<VectoredLine>) {
        let mut progress = 0.0;
        let (blueprint_vectors, color_offset) = if self.animation_state == AnimationState::Stopped {
            (global_design_vectors, 0)
        } else {
            painter.ctx().request_repaint();
            // move to own function?
            let elapsed_time = self.animation_state.elapsed();
            progress = elapsed_time.as_secs_f32()
                / self.fractals[self.fractal_index].animation.cycle_length;

            // This is where the animation happens:
            self.animation_frame_vectors(
                global_design_vectors,
                progress,
                self.fractals[self.fractal_index]
                    .animation
                    .animation_repetition_cycle,
            )
        };

        let fractal = &self.fractals[self.fractal_index];
        let rect = painter.clip_rect();

        let initiator = blueprint_vectors[0];
        let transformations: Vec<LineTransform> = blueprint_vectors[1..]
            .iter()
            .flat_map(|line| {
                let mut line_transforms: Vec<LineTransform> =
                    vec![LineTransform::from_design_vector(&initiator, *line, false)];
                if fractal.mirror {
                    line_transforms
                        .push(LineTransform::from_design_vector(&initiator, *line, true));
                }
                line_transforms
            })
            .collect();

        let shapes = paint_fractal_lines(
            rect,
            &Branch::from_vectored_line(initiator),
            fractal,
            &transformations,
            fractal.depth,
            Some(progress),
            color_offset,
        );

        self.line_count = shapes.len();
        painter.extend(shapes);
    }
}

impl eframe::App for FractalApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Sets the clear color of the window to white
        [1.0, 1.0, 1.0, 1.0]
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let fractal = &mut self.fractals[self.fractal_index];

        fractal.depth = fractal.depth.at_most(max_depth_with_branches(
            // TODO move this to end of design. Add it to paint_fractal as a dbg assert
            MAX_PAINTED_LINE_COUNT,
            fractal.design_lines.len() - 1,
            fractal.mirror,
            fractal.replace_line,
        ));
        let painter = Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );
        let to_screen = RectTransform::from_to(
            Rect::from_center_size(
                pos2(fractal.center.x, fractal.center.y),
                painter.clip_rect().square_proportions() / fractal.zoom,
            ),
            painter.clip_rect(),
        );

        self.design(ui, to_screen, &painter);

        let fractal = &mut self.fractals[self.fractal_index]; // HACK for now. Change after self.animate is coded.
        let global_design_vectors =
            reversible_lines_to_global_line_vectors(&fractal.design_lines, to_screen);
        if self.show_design_only {
            self.paint_design(&painter, &global_design_vectors);
        } else {
            self.animate(&painter, global_design_vectors);
            if let Some((a, b, c)) = self.a_b_c {
                painter.line_segment([a, b], Stroke::new(2.0, Color32::RED));
                painter.line_segment([a, c], Stroke::new(2.0, Color32::GREEN));
            }
        }

        // Make sure we allocate what we used (everything)
        ui.expand_to_include_rect(painter.clip_rect());

        let lines_style = self.fractals[self.fractal_index].lines_style;
        Frame::popup(ui.style())
            .stroke(Stroke::NONE)
            .show(ui, |ui| {
                ui.set_max_width(270.0);
                CollapsingHeader::new("Settings").show(ui, |ui| self.options_ui(ui));
            });
        if lines_style != self.fractals[self.fractal_index].lines_style {
            handle_line_style_change(self);
        }
    }
}
