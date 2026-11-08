mod animation;
mod bench_helpers;
mod design_helpers;
mod design_input;
mod fractals;
pub mod paint_fractal_helpers; // pub for benching check if needed
pub mod structs_and_enums; // pub for benching TODO check if needed
mod tools;

use std::f32::consts::LN_2;

use egui::{
    Button, Color32, NumExt as _, Painter, Pos2, Rect, RichText, Stroke, Ui,
    containers::{CollapsingHeader, Frame},
    emath::{RectTransform, Rot2},
    pos2,
    widgets::Slider,
};

use animation::{
    animation_helpers::find_animation_rotation_center,
    scale_and_rotate_vectored_lines_around_rotation_center,
};
#[expect(unused)]
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
    fractal_app::animation::animation_structs_and_enums::AnimationProgress,
    structs_and_enums::PlayerState,
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
    // ANIMATION
    #[serde(skip)]
    a_b_c: Option<(Pos2, Pos2, Pos2)>, // for animation. HACK TEMP

    // BENCHING
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

        // ---------------------- ANIMATION-----------------------------------------------
        ui.separator();

        let state = self.fractals[self.fractal_index].animation.player_state;
        let at_start = self.fractals[self.fractal_index]
            .animation
            .current_ln_scale()
            <= 0.0;

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    state != PlayerState::PlayingReverseInfinite,
                    Button::new(RichText::new("◀∞").size(18.0)),
                )
                .on_hover_text("Zoom out forever (reverse past the start)")
                .clicked()
            {
                self.fractals[self.fractal_index]
                    .animation
                    .play_reverse_infinite();
            }
            if ui
                .add_enabled(
                    state != PlayerState::PlayingReverse && !at_start,
                    Button::new(RichText::new("◀").size(18.0)),
                )
                .on_hover_text("Play in reverse (stops at the start)")
                .clicked()
            {
                self.fractals[self.fractal_index].animation.play_reverse();
            }
            if ui
                .add_enabled(
                    self.fractals[self.fractal_index].animation.is_playing(),
                    Button::new(RichText::new("⏸").size(18.0)),
                )
                .on_hover_text("Pause")
                .clicked()
            {
                self.fractals[self.fractal_index].animation.pause();
            }
            if ui
                .add_enabled(
                    state != PlayerState::Playing,
                    Button::new(RichText::new("▶").size(18.0)),
                )
                .on_hover_text("Play")
                .clicked()
            {
                self.fractals[self.fractal_index].animation.play();
            }
            if ui
                .add_enabled(
                    state != PlayerState::Stopped
                        || self.fractals[self.fractal_index]
                            .animation
                            .current_ln_scale()
                            != 0.0,
                    Button::new(RichText::new("⏹").size(18.0)),
                )
                .on_hover_text("Stop (reset the animation)")
                .clicked()
            {
                self.fractals[self.fractal_index].animation.stop();
            }
        });
        let fractal = &mut self.fractals[self.fractal_index];

        // NOTE: egui has a built in way to do this, but it's not as straightforward: `from_get_set`
        let mut zpm = fractal.animation.ln_scale_per_second() * 60.0 / LN_2;
        if ui
            .add(egui::Slider::new(&mut zpm, 5.0..=180.0).text("Animation Zooms per minutes"))
            .on_hover_text("After one zoom the fractal doubles in size.")
            .changed()
        {
            fractal.animation.set_ln_scale_per_second(zpm * LN_2 / 60.0);
        }

        // TODO!!!! see project.md: change this to "repeat when showing" (Tool tip: percentage of length showing when repetition can start):
        ui.add(
            Slider::new(&mut fractal.animation.animation_loop_cycle, 0..=10)
                .text("cycle that can be repeated"),
        );

        ui.checkbox(
            &mut fractal.animation.zoom_child_to_parent,
            "Zoom child to parent",
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
    #[expect(clippy::needless_pass_by_value)] // We want to consume the input vector, to force a more functional programming style. This is a design choice, not a performance choice.
    fn animation_paint_vectors_and_progress(
        &mut self,
        gdvs: Vec<VectoredLine>, // global design vectors, consumed.
    ) -> (Vec<VectoredLine>, Option<AnimationProgress>) {
        // return owned vector, because we consumed the input vector.
        let animation = &self.fractals[self.fractal_index].animation;
        let cycle_angle = gdvs[0].vec.angle() - gdvs[1].vec.angle();
        let cycle_scale = gdvs[0].vec.length() / gdvs[1].vec.length();

        let progress = AnimationProgress::new_from_ln_scale(
            animation.current_ln_scale(),
            cycle_scale as f64, // ugly
            animation,
        );

        // TODO: better naming
        // HACK, check if there is a generator line. TODO!!!!!

        let b = gdvs[1].pos;
        let c = gdvs[0].pos;
        let rotation_center = find_animation_rotation_center(b, c, cycle_angle, cycle_scale);
        // To VISUALIZE the a b c triangle:
        self.a_b_c = Some((rotation_center.unwrap_or(Pos2::new(-2.0, -1.0)), b, c));

        let scale = cycle_scale.powf(progress.elapsed_wrapped);
        let rotattion_angle = if animation.zoom_child_to_parent {
            progress.elapsed_wrapped * cycle_angle
        } else {
            progress.loop_repetitions as f32 * -cycle_angle
        };
        let rotation = Rot2::from_angle(rotattion_angle);
        // HACK, check if there is a generator line. TODO!!!!!

        (
            scale_and_rotate_vectored_lines_around_rotation_center(
                &gdvs,
                rotation_center.unwrap_or(Pos2::new(-2.0, -1.0)), // UGLY HACK, TODO
                scale,
                rotation,
            ),
            Some(progress),
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

    fn paint_fractal(
        &mut self,
        painter: &Painter,
        global_design_vectors: &[VectoredLine],
        animation_progress: Option<AnimationProgress>,
    ) {
        let fractal = &self.fractals[self.fractal_index];
        let rect = painter.clip_rect();

        let initiator = global_design_vectors[0];
        let transformations: Vec<LineTransform> = global_design_vectors[1..]
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
            animation_progress,
        );

        self.line_count = shapes.len();
        painter.extend(shapes);
    }
}

impl eframe::App for FractalApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // no longer needed with Claude rfactor to use egui.stable_dt
        // self.fractals[self.fractal_index]
        //     .animation
        //     .update_current_ln_scale();
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Sets the clear color of the window to white
        [1.0, 1.0, 1.0, 1.0]
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let fractal = &mut self.fractals[self.fractal_index];

        let dt = ui.ctx().input(|i| i.stable_dt);
        fractal.animation.update(dt);

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
            let (paint_vectors, animation_progress) = if fractal.animation.is_stopped() {
                (global_design_vectors, None)
            } else {
                if fractal.animation.is_playing() {
                    painter.ctx().request_repaint();
                }
                self.animation_paint_vectors_and_progress(global_design_vectors)
            };
            self.paint_fractal(&painter, &paint_vectors, animation_progress);
        }

        // Make sure we allocate what we used (everything)
        ui.expand_to_include_rect(painter.clip_rect());

        // HACK: Move lines_style check to options_ui. See Claude Rust group "Checking line style changes in Rust" tab.
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
