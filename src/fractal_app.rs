mod design_helpers;
mod paint_fractal_helpers;
mod states;
mod structs;
mod tools;

use design_helpers::{
    closest_handle, closest_line, design_lines_to_global_design_vectors,
    paint_directed_line_segment,
};
use egui::{
    Color32, NumExt as _, Painter, Pos2, Rect, Shape, Stroke, Ui,
    containers::{CollapsingHeader, Frame},
    emath::{self},
    pos2,
    widgets::Slider,
};
use paint_fractal_helpers::line_color;
use structs::{LineTransform, LinesStyle, Node, State, VectoredDesignLine};
use tools::max_depth_with_branches;

use crate::fractal_app::structs::DesignLine;

const MAX_PAINTED_LINE_COUNT: usize = (1 << 18) + 1; // 2 to the power of 18 + 1. HARDCODED

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FractalApp {
    states: Vec<State>,
    si: usize, // state index
    fine_tune: bool,
    line_count: usize,
    dragged_line_end_point: Option<[usize; 2]>, // Add option for incorrect drag. Now it catches an endpoint when dragging over it, after starting in the middle of nowhere :)
    show_design_only: bool,
}

impl Default for FractalApp {
    fn default() -> Self {
        Self {
            states: vec![State {
                mirror: false,
                rainbow: false,
                design_line_count: 1,
                design_lines: vec![
                    DesignLine {
                        line: [Pos2::new(0.0, 0.0), Pos2::new(0.0, -1.0)],
                        reversed: false,
                    },
                    DesignLine {
                        line: [Pos2::new(0.0, -1.0), Pos2::new(0.5, -1.5)],
                        reversed: false,
                    },
                    DesignLine {
                        line: [Pos2::new(0.0, -1.0), Pos2::new(0.0, -1.70)],
                        reversed: false,
                    },
                    DesignLine {
                        line: [Pos2::new(0.0, -1.0), Pos2::new(-0.5, -1.5)],
                        reversed: false,
                    },
                    DesignLine {
                        line: [Pos2::new(0.0, -1.0), Pos2::new(-0.5, -0.5)],
                        reversed: false,
                    },
                ],
                replace_line: false,
                lines_style: LinesStyle::Free,
                zoom: 0.18,
                center: pos2(0.0, -2.5),
                start_line_width: 2.5, // TODO strangely global screen coords width... prob OK. Has to be visible
                fixed_final_line_width: 1.0,
                depth: 9,
            }],
            si: 0,
            line_count: 0,
            show_design_only: false,
            fine_tune: false,
            dragged_line_end_point: None,
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
        let max_depth = max_depth_with_branches(
            MAX_PAINTED_LINE_COUNT,
            self.states[self.si].design_line_count,
            self.states[self.si].mirror,
        );
        ui.label(format!("Painted line count: {}", self.line_count));
        ui.checkbox(
            &mut self.states[self.si].replace_line,
            "Replace parent with children",
        );
        ui.checkbox(&mut self.states[self.si].mirror, "Mirror");
        ui.checkbox(&mut self.states[self.si].rainbow, "Rainbow");
        let iterator_count = &self.states[self.si].design_lines.len() - 1;
        ui.add(
            Slider::new(
                &mut self.states[self.si].design_line_count,
                1..=iterator_count,
            )
            .text("Design line count"),
        );
        ui.radio_value(
            &mut self.states[self.si].lines_style,
            LinesStyle::Free,
            "Free",
        );
        ui.radio_value(
            &mut self.states[self.si].lines_style,
            LinesStyle::Tree,
            "Tree",
        );
        ui.radio_value(
            &mut self.states[self.si].lines_style,
            LinesStyle::Loop,
            "Loop",
        );
        ui.add(Slider::new(&mut self.states[self.si].zoom, 0.001..=5.0).text("zoom"));
        if self.states[self.si].replace_line {
            ui.add(
                Slider::new(&mut self.states[self.si].fixed_final_line_width, 0.0..=7.0)
                    .text("Final line width"),
            )
        } else {
            ui.add(
                Slider::new(&mut self.states[self.si].start_line_width, 0.0..=7.0)
                    .text("Start line width"),
            )
        };
        ui.add(Slider::new(&mut self.states[self.si].depth, 0..=max_depth).text("depth"));

        egui::reset_button(ui, self, "Reset");

        ui.add(egui::github_link_file!(
            "https://github.com/mjvandermeulen/egui-fractals/blob/main/",
            "Source code."
        ));
    }

    fn design(&mut self, ui: &Ui, painter: &Painter) -> Vec<VectoredDesignLine> {
        let to_screen = emath::RectTransform::from_to(
            Rect::from_center_size(
                pos2(self.states[self.si].center.x, self.states[self.si].center.y),
                painter.clip_rect().square_proportions() / self.states[self.si].zoom,
            ),
            painter.clip_rect(),
        );
        let from_screen = to_screen.inverse();

        let rect = painter.clip_rect();
        let id = ui.make_persistent_id("design_painter_interaction");

        // Keyboard Input

        // https://github.com/emilk/egui/discussions/1464 -> if. fine tuned with gemini. Maarten.
        if ui.ctx().memory(|mem| mem.focused()).is_none() {
            let max_depth = max_depth_with_branches(
                MAX_PAINTED_LINE_COUNT,
                self.states[self.si].design_line_count,
                self.states[self.si].mirror,
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
                                self.states[self.si].depth = max_depth;
                            } else if number == 8 {
                                // NOTE: max_depth could be < 8, so you can't clamp(8, max_depth);
                                self.states[self.si].depth =
                                    (max_depth / 2).at_least(8).clamp(0, max_depth);
                            } else {
                                self.states[self.si].depth = number.clamp(0, max_depth);
                            }
                        }
                    }
                }
            });
            // up and down arrows
            if self.states[self.si].depth > 0 //clamping doesn't avoid a usize overflow soon enough
                && ui.input_mut(|i| i.key_pressed(egui::Key::ArrowDown))
            {
                self.states[self.si].depth = (self.states[self.si].depth - 1).clamp(0, max_depth);
            }
            if ui.input_mut(|i| i.key_pressed(egui::Key::ArrowUp)) {
                self.states[self.si].depth = (self.states[self.si].depth + 1).clamp(0, max_depth);
            }
        }

        if ui.input(|i| i.modifiers.shift_only()) {
            // shift down
            if ui.input(|i| i.key_pressed(egui::Key::D)) {
                self.show_design_only = !self.show_design_only;
            }
        } else {
            // shift up
            if ui.input(|i| i.key_down(egui::Key::D)) {
                self.show_design_only = true;
            } else if ui.input(|i| i.key_released(egui::Key::D)) {
                self.show_design_only = false;
            }
        }
        self.fine_tune = ui.input(|i| i.modifiers.ctrl);

        // Mouse input

        let scroll_response = ui.interact(rect, id, egui::Sense::hover());
        if scroll_response.hovered() {
            ui.input(|input| {
                for event in &input.events {
                    if let egui::Event::MouseWheel { delta, .. } = event {
                        // 'delta.y' is the vertical scroll (Mac trackpad two-finger vertical)
                        // 'delta.x' is the horizontal scroll (Mac trackpad two-finger horizontal)
                        self.states[self.si].center += from_screen.scale().x * (-1.0 * *delta);
                    } else {
                        let zoom_delta = input.zoom_delta();
                        if zoom_delta != 1.0 {
                            self.states[self.si].zoom *= zoom_delta;
                        }
                    }
                }
            });
        }

        let click_and_drag_response = ui.interact(rect, id, egui::Sense::click_and_drag());
        if click_and_drag_response.is_pointer_button_down_on() {
            // is_pointer_down vs dragged: see tool tip on `dragged`. We don't want a delay.
            if self.dragged_line_end_point.is_none()
                && let Some(screenpos) = click_and_drag_response.interact_pointer_pos()
            {
                let local_pos = from_screen * screenpos;
                self.dragged_line_end_point = closest_handle(
                    local_pos,
                    &self.states[self.si].design_lines
                        [..self.states[self.si].design_line_count + 1],
                    &self.states[self.si].lines_style,
                );
            }
            if let Some([line, end]) = self.dragged_line_end_point {
                let tuning_ratio = if self.fine_tune { 0.02 } else { 1.0 };
                let new_point = from_screen
                    * (to_screen * self.states[self.si].design_lines[line].line[end]
                        + tuning_ratio * click_and_drag_response.drag_delta());
                if self.states[self.si].lines_style == LinesStyle::Loop {
                    debug_assert_ne!(
                        end, 0,
                        "Loop style expects that the start point of a line can not be dragged"
                    );
                    self.states[self.si].design_lines[line].line[1] = new_point;
                    let next_line_index = (line + 1) % (self.states[self.si].design_line_count + 1);
                    self.states[self.si].design_lines[next_line_index].line[0] = new_point;
                } else if self.states[self.si].lines_style == LinesStyle::Tree {
                    self.states[self.si].design_lines[line].line[end] = new_point;
                    if line == 0 && end == 1 {
                        self.states[self.si]
                            .design_lines
                            .iter_mut()
                            .skip(1)
                            .for_each(|d_line| d_line.line[0] = new_point);
                    }
                } else {
                    self.states[self.si].design_lines[line].line[end] = new_point;
                }
            }
        } else {
            self.dragged_line_end_point = None;
            if click_and_drag_response.double_clicked()
                && let Some(screen_pos) = ui.input(|i| i.pointer.hover_pos())
            {
                let pos = from_screen * screen_pos;
                if let Some(i) = closest_line(pos, &self.states[self.si].design_lines) {
                    self.states[self.si].design_lines[i].reversed =
                        !self.states[self.si].design_lines[i].reversed;
                }
            }
        }

        design_lines_to_global_design_vectors(
            &self.states[self.si].design_lines[..self.states[self.si].design_line_count + 1],
            to_screen,
        )
    }

    fn paint_design(&self, painter: &Painter, design_vectors: &[VectoredDesignLine]) {
        design_vectors.iter().enumerate().for_each(|(i, vec)| {
            let (width, color) = if i == 0 {
                (self.states[self.si].start_line_width * 1.5, Color32::RED)
            } else {
                (self.states[self.si].start_line_width, Color32::ORANGE)
            };
            paint_directed_line_segment(painter, vec, width, color);
        });
    }

    fn paint_fractal(&mut self, painter: &Painter, vectored_design_lines: &[VectoredDesignLine]) {
        debug_assert!(
            self.states[self.si].depth
                <= max_depth_with_branches(
                    MAX_PAINTED_LINE_COUNT,
                    vectored_design_lines.len() - 1,
                    self.states[self.si].mirror
                ),
            "self.states[self.si].depth = {}, max_depth_with_branches(...) = {}",
            self.states[self.si].depth,
            max_depth_with_branches(
                MAX_PAINTED_LINE_COUNT,
                vectored_design_lines.len() - 1,
                self.states[self.si].mirror
            )
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

        let base = vectored_design_lines[0];
        let transformations: Vec<LineTransform> = vectored_design_lines[1..]
            .iter()
            .flat_map(|line| {
                let mut line_transforms: Vec<LineTransform> =
                    vec![LineTransform::from_design_vector(&base, *line, false)];
                if self.states[self.si].mirror {
                    line_transforms.push(LineTransform::from_design_vector(&base, *line, true));
                }
                line_transforms
            })
            .collect();
        let base_line_width = if self.states[self.si].replace_line {
            self.states[self.si].fixed_final_line_width
        } else {
            self.states[self.si].start_line_width
        };
        if !self.states[self.si].replace_line || self.states[self.si].depth == 0 {
            paint_line(
                [base.pos, base.pos + base.vec],
                line_color(0, self.states[self.si].rainbow),
                base_line_width,
            );
        }

        // CORE paint_fractal loop:
        let base_length = base.vec.length();
        let mut nodes = vec![Node {
            pos: base.pos,
            vec: base.vec,
        }];

        let mut new_nodes = Vec::new();
        for depth in 1..self.states[self.si].depth + 1 {
            let color = line_color(depth, self.states[self.si].rainbow);

            if depth < self.states[self.si].depth {
                new_nodes.clear();
                new_nodes.reserve(nodes.len() * 2);
            }

            // iterate over stored parent nodes
            //  create a new node per transformation and paint the line in it
            //  if we're not at the max depth, store the new node for the next iteration
            for parent_node in &nodes {
                for &transform in &transformations {
                    let paint_a = parent_node.pos + transform.base_rot * parent_node.vec;
                    let paint_vec = transform.rot * parent_node.vec;
                    let paint_b = paint_a + paint_vec;
                    let painted_node = Node {
                        pos: paint_a,
                        vec: paint_vec,
                    };

                    if self.states[self.si].replace_line {
                        if depth == self.states[self.si].depth {
                            paint_line(
                                [paint_a, paint_b],
                                color,
                                self.states[self.si].fixed_final_line_width,
                            );
                        }
                    } else {
                        paint_line(
                            [paint_a, paint_b],
                            color,
                            (painted_node.vec.length() / base_length)
                                * self.states[self.si].start_line_width,
                        );
                    }
                    if depth < self.states[self.si].depth {
                        new_nodes.push(painted_node);
                    }
                }
            }

            std::mem::swap(&mut nodes, &mut new_nodes);
        }
        self.line_count = shapes.len();
        // log::info!("self.depth = {}", self.depth);
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
        self.states[self.si].depth = self.states[self.si].depth.at_most(max_depth_with_branches(
            // TODO move this to end of design. Add it to paint_fractal as a dbg assert
            MAX_PAINTED_LINE_COUNT,
            self.states[self.si].design_line_count,
            self.states[self.si].mirror,
        ));
        let painter = Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );

        let design_vectors = self.design(ui, &painter);
        if self.show_design_only {
            self.paint_design(&painter, &design_vectors);
        } else {
            self.paint_fractal(&painter, &design_vectors);
        }

        // if let Some(line) = self.hovered_design_line {
        //     paint_directed_line_segment(&painter, dvec, width, color);
        // }

        // Make sure we allocate what we used (everything)
        ui.expand_to_include_rect(painter.clip_rect());

        Frame::popup(ui.style())
            .stroke(Stroke::NONE)
            .show(ui, |ui| {
                ui.set_max_width(270.0);
                CollapsingHeader::new("Settings").show(ui, |ui| self.options_ui(ui));
            });
    }
}
