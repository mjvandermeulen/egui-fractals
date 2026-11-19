mod animation;
mod bench_helpers;
mod design_helpers;
mod fractals;
mod paint_fractal_helpers; // pub for benching check if needed
mod structs_and_enums; // pub for benching TODO check if needed
mod tools;
mod ui;

use egui::{
    Button, Color32, NumExt as _, Painter, Pos2, Rect, RichText, Stroke, Ui,
    containers::{CollapsingHeader, Frame},
    emath::{RectTransform, Rot2},
    pos2,
    widgets::Slider,
};
use std::f32::consts::LN_2;

use animation::{
    animation_helpers::scale, animation_helpers::spiral_similarity_center,
    animation_structs_and_enums::AnimationProgress,
    scale_and_rotate_vectored_lines_around_rotation_center,
};
use design_helpers::{
    handle_line_style_change, paint_directed_line_segment, reversible_lines_to_global_line_vectors,
};
use fractals::fractals;
use paint_fractal_helpers::paint_fractal_lines;
use structs_and_enums::{
    Branch, Fractal, LineHandles, LineTransform, LinesStyle, ReversibleLine, VectoredLine,
};
use tools::max_depth_with_branches;
use ui::{handle_keyboard_input, handle_mouse_input};

#[expect(unused)]
use bench_helpers::save_bench_prep_parallel_struct_to_json;

use crate::fractal_app::animation::animation_structs_and_enums::Triangle;

// TODO: find the error that pops op when the vec gets too large.
const MAX_PAINTED_LINE_COUNT: usize = (1 << 18) + 100; // 2 to the power of 18 + 1. HARDCODED

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FractalApp {
    fractals: Vec<Fractal>,
    fractal_index: usize,
    #[serde(skip)]
    latest_painted_line_count: usize, // line count of the most recently painted fractal.
    #[serde(skip)]
    fine_tune: f32, //  1.0 (default), 10.0 times <Alt> 100.0 times <Ctrl> and 1000.0 times <Ctrl-Alt>
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

    // ANIMATION (temp values)
    #[serde(skip)]
    spiral_similarity_trangle: Option<Triangle>, // for animation. HACK TEMP

    // BENCHING
    #[serde(skip)]
    bench_prep: bool,
}

impl Default for FractalApp {
    fn default() -> Self {
        Self {
            fractals: fractals(),
            fractal_index: 0,
            latest_painted_line_count: 0,
            show_design_only: false,
            fine_tune: 1.0,
            dragged_handles: None,
            new_line_key_down: false,
            trash_line_key_down: false,
            hovered_line: None,
            spiral_similarity_trangle: None,
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

    fn stats_ui(&self, ui: &mut Ui) {
        ui.label(format!(
            "Painted line count: {}",
            self.latest_painted_line_count
        ));
    }
    #[expect(clippy::too_many_lines)]
    fn options_ui(&mut self, ui: &mut Ui, animation_progress: Option<AnimationProgress>) {
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

        ui.horizontal(|ui| {
            let fractal = &mut self.fractals[self.fractal_index];

            if ui
                .add_enabled(
                    fractal
                        .animation
                        .reverse_infinite_is_enabled(animation_progress),
                    Button::new(RichText::new("◀∞").size(18.0)),
                )
                .on_hover_text("Zoom out forever (reverse past the start)")
                .clicked()
            {
                fractal.animation.play_reverse_infinite();
            }
            if ui
                .add_enabled(
                    fractal.animation.reverse_is_enabled(animation_progress),
                    Button::new(RichText::new("◀").size(18.0)),
                )
                .on_hover_text("Play in reverse (stops at the start)")
                .clicked()
            {
                fractal.animation.play_reverse();
            }
            if ui
                .add_enabled(
                    fractal.animation.pause_is_enabled(),
                    Button::new(RichText::new("⏸").size(18.0)),
                )
                .on_hover_text("Pause")
                .clicked()
            {
                fractal.animation.pause();
            }
            if ui
                .add_enabled(
                    fractal.animation.play_forward_is_enabled(),
                    Button::new(RichText::new("▶").size(18.0)),
                )
                .on_hover_text("Play")
                .clicked()
            {
                fractal.animation.play();
            }
            if ui
                .add_enabled(
                    fractal.animation.stop_is_enabled(),
                    Button::new(RichText::new("⏹").size(18.0)),
                )
                .on_hover_text("Stop (reset the animation)")
                .clicked()
            {
                fractal.animation.stop();
            }
        });
        let fractal = &mut self.fractals[self.fractal_index];

        // NOTE: egui has a built in way to do this, but it's not as straightforward: `from_get_set`
        let mut zpm = fractal.animation.ln_scale_per_second * 60.0 / LN_2;
        if ui
            .add(egui::Slider::new(&mut zpm, 5.0..=180.0).text("Animation Zooms per minutes"))
            .on_hover_text("After one zoom the fractal doubles in size.")
            .changed()
        {
            fractal.animation.ln_scale_per_second = zpm * LN_2 / 60.0;
        }

        // TODO!!!! see project.md: change this to "repeat when showing" (Tool tip: percentage of length showing when repetition can start):
        ui.add(
            Slider::new(&mut fractal.animation.loop_cycle, 0..=10)
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

    fn handle_custom_ui(&mut self, ui: &Ui, to_screen: RectTransform, painter: &Painter) {
        handle_keyboard_input(ui, self);
        handle_mouse_input(ui, self, to_screen, painter.clip_rect());
    }
    // #[expect(clippy::needless_pass_by_value)] // NO LONGER NEEDED, BECAUSE OF NEW CHECK WITH EARLY RETURN LEARN. We want to consume the input vector, to force a more functional programming style. This is a design choice, not a performance choice.
    fn animation_paint_vectors(
        &mut self,
        gdvs: Vec<VectoredLine>, // global design vectors, consumed.
        animation_progress: Option<AnimationProgress>,
    ) -> Vec<VectoredLine> {
        let Some(progress) = animation_progress else {
            return gdvs;
        };
        if gdvs.is_empty() {
            log::warn!("The global design vectors Vec is empty. This should never happen");
            return gdvs;
        }
        #[expect(clippy::bool_to_int_with_if)] // it's not really a bool to int here
        let generator_index = if gdvs.len() == 1 { 0 } else { 1 };
        // return owned vector, because we consumed the input vector.
        let animation = &mut self.fractals[self.fractal_index].animation;
        let cycle_angle = gdvs[0].vec.angle() - gdvs[generator_index].vec.angle();
        let cycle_scale = gdvs[0].vec.length() / gdvs[generator_index].vec.length();

        let point = gdvs[generator_index].pos; // pre image: P of the spiral similarity
        let point_image = gdvs[0].pos; // image: P' of the spiral similarity
        let center = spiral_similarity_center(point, point_image, cycle_angle, cycle_scale)
            .unwrap_or(Pos2::new(0.0, 0.0)); // NOTE: This is actually OK: None means: identity tranformation: So pick your center at will.
        // To VISUALIZE:
        // Currently unused! Not even commented out code.
        self.spiral_similarity_trangle = Some(Triangle::new(center, point, point_image));

        let scale = cycle_scale.powf(progress.elapsed_wrapped);
        let rotattion_angle = if animation.zoom_child_to_parent {
            progress.elapsed_wrapped * cycle_angle
        } else {
            progress.loop_repetitions as f32 * -cycle_angle
        };
        let rotation = Rot2::from_angle(rotattion_angle);

        scale_and_rotate_vectored_lines_around_rotation_center(&gdvs, center, scale, rotation)
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

        self.latest_painted_line_count = shapes.len();
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
        let delta_time = ui.ctx().input(|i| i.stable_dt);
        let painter = Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );

        let fractal = &mut self.fractals[self.fractal_index];
        fractal.animation.update(delta_time);

        let to_screen = RectTransform::from_to(
            Rect::from_center_size(
                pos2(fractal.center.x, fractal.center.y),
                painter.clip_rect().square_proportions() / fractal.zoom,
            ),
            painter.clip_rect(),
        );

        self.handle_custom_ui(ui, to_screen, &painter);

        // --- clip user input ---
        let fractal = &mut self.fractals[self.fractal_index];
        fractal.depth = fractal.depth.at_most(max_depth_with_branches(
            MAX_PAINTED_LINE_COUNT,
            fractal.design_lines.len() - 1,
            fractal.mirror,
            fractal.replace_line,
        ));

        // --- paint (and animate) ---
        let mut animation_progress: Option<AnimationProgress> = None;
        let global_design_vectors =
            reversible_lines_to_global_line_vectors(&fractal.design_lines, to_screen);
        if self.show_design_only {
            self.paint_design(&painter, &global_design_vectors);
        } else {
            let paint_vectors = if fractal.animation.is_stopped() {
                global_design_vectors
            } else {
                if fractal.animation.not_paused_or_stopped() {
                    painter.ctx().request_repaint();
                }
                let generator_index: usize = usize::from(global_design_vectors.len() != 1);
                let cycle_scale = scale(
                    global_design_vectors[generator_index],
                    global_design_vectors[0],
                );
                animation_progress = Some(AnimationProgress::new_from_cycle_scale(
                    cycle_scale as f64,
                    &fractal.animation,
                ));
                self.animation_paint_vectors(global_design_vectors, animation_progress)
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
                CollapsingHeader::new("Stats").show(ui, |ui| self.stats_ui(ui));
            });
        Frame::popup(ui.style())
            .stroke(Stroke::NONE)
            .show(ui, |ui| {
                ui.set_max_width(270.0);
                CollapsingHeader::new("Settings")
                    .show(ui, |ui| self.options_ui(ui, animation_progress));
            });
        if lines_style != self.fractals[self.fractal_index].lines_style {
            handle_line_style_change(self);
        }
    }
}
