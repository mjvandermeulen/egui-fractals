mod animation;
mod design_helpers;
mod design_input;
mod fractals;
mod paint_fractal_helpers;
mod structs_and_enums;
mod tools;

use std::time::Instant;

use design_helpers::{paint_directed_line_segment, reversible_lines_to_global_line_vectors};
use egui::{
    Button, Color32, NumExt as _, Painter, Pos2, Rect, Shape, Stroke, Ui,
    containers::{CollapsingHeader, Frame},
    emath::RectTransform,
    pos2,
    widgets::Slider,
};
use paint_fractal_helpers::line_color;
use structs_and_enums::{Fractal, LineTransform, LinesStyle, Node, VectoredLine};
use tools::max_depth_with_branches;

use crate::fractal_app::{
    animation::{animation_tools::find_animation_rotation_center, scale_and_rotate_vectored_lines},
    design_helpers::handle_line_style_change,
    design_input::{handle_keyboard_input, handle_mouse_input},
    fractals::fractals,
    structs_and_enums::{LineHandles, ReversibleLine},
};

const MAX_PAINTED_LINE_COUNT: usize = (1 << 18) + 100; // 2 to the power of 18 + 1. HARDCODED

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FractalApp {
    fractals: Vec<Fractal>,
    fractal_index: usize,
    #[serde(skip)]
    line_count: usize,
    #[serde(skip)]
    fine_tune: bool, // turn into f32 for normal 1.0, ten times (Alt) 10.0 and 100 times (Ctrl) 100.0
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

    // TODO!!! put all animation stuff in a struct.
    #[serde(skip)]
    animation_start: Option<Instant>,
    // TODO!!! progress: f32, // for animation. instead of local, so you can pause and resume.
    #[serde(skip)]
    animation_cycle: usize, // one cycle is when a generator line is animated to take exactly the place of the initiator.
    #[serde(skip)]
    new_cycle: bool,
    #[serde(skip)]
    animation_repetition_cycle: Option<usize>, // the cycle that can be repeated to make a continuous animation.
    #[serde(skip)]
    animation_cycle_start_limited_depth_paint_count: usize,
    #[serde(skip)]
    a_b_c: Option<(Pos2, Pos2, Pos2)>, // for animation. HACK TEMP
}

impl Default for FractalApp {
    fn default() -> Self {
        Self {
            fractals: fractals(),
            fractal_index: 0,
            line_count: 0,
            show_design_only: false,
            fine_tune: false,
            dragged_handles: None,
            new_line_key_down: false,
            trash_line_key_down: false,
            hovered_line: None,
            animation_start: None,
            animation_cycle: 0,
            new_cycle: true, // works, but semi ugly.
            animation_repetition_cycle: None,
            animation_cycle_start_limited_depth_paint_count: 0,
            a_b_c: None,
        }
    }
}

impl FractalApp {
    /// Called once before the first frame.
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

    fn options_ui(&mut self, ui: &mut Ui) {
        egui::ComboBox::from_label("Select starter Fractal")
            .selected_text(&self.fractals[self.fractal_index].name)
            .show_ui(ui, |ui| {
                for (index, fractal) in self.fractals.iter().enumerate() {
                    ui.selectable_value(&mut self.fractal_index, index, &fractal.name);
                }
            });

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

        let max_depth = max_depth_with_branches(
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
            )
        } else {
            ui.add(
                Slider::new(&mut fractal.initiator_length_width_ratio, 25.0..=100.0)
                    .text("length/width ratio"),
            )
        };
        ui.add(Slider::new(&mut fractal.depth, 0..=max_depth).text("depth"));

        egui::reset_button(ui, self, "Full Reset"); // NOTE: will not looked disabled, because of self.line_count

        // ---------------------------------------------------------------------
        ui.separator();

        let animate_text = if self.animation_start.is_some() {
            "Stop Animation"
        } else {
            "Start Animation"
        };
        if ui.add_enabled(true, Button::new(animate_text)).on_hover_text(if self.animation_start.is_some() {
            "Stop animating the fractal. TODO!!! Press space to pause/resume. Press R to reset animation."
        } else {
            "Start animating the fractal. TODO!!! Press space to pause/resume. Press R to reset animation."
        }).clicked(){
            self.animation_start = if self.animation_start.is_some() {
                None
            } else {
                Some(Instant::now())
            };
        }
        // if let Some(animation) = &mut fractal.animation { TODO!!!!
        // ui.add(
        //     Slider::new(&mut fractal.animation.length, 0.5..=30.0)
        //         .text("Animation length (seconds)"),
        // );

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

    fn animation_frame_vectors(
        &mut self,
        gdvs: &[VectoredLine], // global design vectors
        start: Instant,
    ) -> Vec<VectoredLine> {
        let cycle_length = self.fractals[self.fractal_index].animation.length;
        let progress = animation::animation_tools::animation_progress(
            start,
            cycle_length,
            self.animation_repetition_cycle,
        );
        let cycle: usize = progress.floor() as usize;
        self.new_cycle = cycle != self.animation_cycle;
        self.animation_cycle = cycle;

        let cycle_angle = gdvs[0].vec.angle() - gdvs[1].vec.angle();
        let cycle_scale = gdvs[0].vec.length() / gdvs[1].vec.length();
        // HACK, check if there is a generator line. TODO!!!!!

        let b = gdvs[1].pos;
        let c = gdvs[0].pos;

        let rotation_center = find_animation_rotation_center(b, c, cycle_angle, cycle_scale);
        self.a_b_c = Some((rotation_center.unwrap_or(Pos2::new(-2.0, -1.0)), b, c));

        scale_and_rotate_vectored_lines(
            gdvs,
            rotation_center.unwrap_or(Pos2::new(-2.0, -1.0)), // UGLY HACK, TODO!!!!!
            cycle_angle,
            cycle_scale,
            progress,
        )
    }

    fn paint_design(&self, painter: &Painter, design_vectors: &[VectoredLine]) {
        let lw_ratio = self.fractals[self.fractal_index].initiator_length_width_ratio;
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
            paint_directed_line_segment(painter, vec, lw_ratio, color);
        });
    }
    #[expect(clippy::too_many_lines)] // TODO
    fn paint_fractal(&mut self, painter: &Painter, vectored_design_lines: &[VectoredLine]) {
        let fractal = &self.fractals[self.fractal_index];
        let started_next_cycle = self.animation_start.is_some()
            && self.animation_repetition_cycle.is_none()
            && self.new_cycle;
        let paint_depth = if started_next_cycle {
            fractal.depth - 1
        } else {
            fractal.depth
        };
        let max_depth = max_depth_with_branches(
            MAX_PAINTED_LINE_COUNT,
            vectored_design_lines.len() - 1,
            fractal.mirror,
            fractal.replace_line,
        );
        debug_assert!(
            paint_depth <= max_depth,
            "paint_depth = {paint_depth}, max_depth_with_branches(...) = {max_depth}"
        );
        let mut shapes: Vec<Shape> = Vec::new();
        let rect = painter.clip_rect();
        let mut paint_line = |points: [Pos2; 2], color: Color32, width: f32| {
            let line: [Pos2; 2] = [points[0], points[1]];
            // culling
            if rect.intersects(Rect::from_two_pos(line[0], line[1])) {
                shapes.push(Shape::line_segment(line, (width, color)));
            }
        };

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
        if !fractal.replace_line || paint_depth == 0 {
            paint_line(
                [initiator.pos, initiator.pos + initiator.vec],
                line_color(0, fractal.rainbow),
                initiator.vec.length() / fractal.initiator_length_width_ratio,
            );
        }

        // CORE paint_fractal loop:
        let mut nodes = vec![Node {
            pos: initiator.pos,
            vec: initiator.vec,
        }];

        let mut new_nodes = Vec::new();
        for depth in 1..paint_depth + 1 {
            let color = line_color(depth, fractal.rainbow);

            if depth < paint_depth {
                new_nodes.clear();
                new_nodes.reserve(nodes.len() * 2);
            }

            // iterate over stored parent nodes
            //  create a new node per transformation and paint the line in it
            //  if we're not at the max depth, store the new node for the next iteration

            // the nesting of the node loop inside the transformations loop is purely for speed
            //   it is (just) noticibly faster with a 1 branch depth 17 mirrorred tree
            // Feel free to read it the other way around.
            for &transform in &transformations {
                for parent_node in &nodes {
                    let paint_a = parent_node.pos + transform.base_rot * parent_node.vec;
                    let paint_vec = transform.rot * parent_node.vec;
                    let paint_b = paint_a + paint_vec;
                    let painted_node = Node {
                        pos: paint_a,
                        vec: paint_vec,
                    };

                    if fractal.replace_line {
                        if depth == paint_depth {
                            paint_line([paint_a, paint_b], color, fractal.fixed_final_line_width);
                        }
                    } else {
                        paint_line(
                            [paint_a, paint_b],
                            color,
                            painted_node.vec.length() / fractal.initiator_length_width_ratio,
                        );
                    }
                    if depth < paint_depth {
                        new_nodes.push(painted_node);
                    }
                }
            }

            std::mem::swap(&mut nodes, &mut new_nodes);
        }

        if started_next_cycle {
            // if the last frame of the previous cycle has the same painted line count as the first (depth limited) frame,
            //   we can start to repeat this cycle
            if self.line_count == self.animation_cycle_start_limited_depth_paint_count {
                self.animation_repetition_cycle = Some(self.animation_cycle);
            }
            log::info!(
                "Cycle {} - Previous {} - Painted {} --- fractal depth {} - paint depth {}",
                self.animation_cycle,
                self.line_count,
                shapes.len(),
                fractal.depth,
                paint_depth
            );

            self.animation_cycle_start_limited_depth_paint_count = shapes.len();
        }
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
        if self.animation_start.is_some() {
            ui.ctx().request_repaint();
        }
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
            let blueprint_vectors: Vec<VectoredLine> = if let Some(start) = self.animation_start {
                self.animation_frame_vectors(&global_design_vectors, start)
            } else {
                global_design_vectors
            };

            self.paint_fractal(&painter, &blueprint_vectors);

            // if self.a_b_c.is_some() {
            //     let (a, b, c) = self
            //         .a_b_c
            //         .expect("We can expect a_b_c to be Some, because we just checked it is Some");
            //     painter.line_segment([a, b], Stroke::new(2.0, Color32::RED));
            //     painter.line_segment([a, c], Stroke::new(2.0, Color32::GREEN));
            // }
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
