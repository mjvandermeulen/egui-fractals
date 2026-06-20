mod design_helpers;
mod design_input;
mod fractals;
mod paint_fractal_helpers;
mod structs;
mod tools;

use design_helpers::{
    closest_handle, closest_line, design_lines_to_global_design_vectors,
    paint_directed_line_segment,
};
use egui::{
    Button, Color32, NumExt as _, Painter, Pos2, Rect, Shape, Stroke, Ui,
    containers::{CollapsingHeader, Frame},
    emath::{self},
    pos2,
    widgets::Slider,
};
use paint_fractal_helpers::line_color;
use structs::{Fractal, LineTransform, LinesStyle, Node, VectoredDesignLine};
use tools::max_depth_with_branches;

use crate::fractal_app::{
    design_input::handle_keyboard_input, fractals::fractals, structs::DesignLine,
};

const MAX_PAINTED_LINE_COUNT: usize = (1 << 18) + 100; // 2 to the power of 18 + 1. HARDCODED

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FractalApp {
    fractals: Vec<Fractal>,
    fractal_index: usize,
    fine_tune: bool,
    line_count: usize,
    dragged_line_end_point: Option<[usize; 2]>, // Add option for incorrect drag. Now it catches an endpoint when dragging over it, after starting in the middle of nowhere :)
    show_design_only: bool,
}

impl Default for FractalApp {
    fn default() -> Self {
        Self {
            fractals: fractals(),
            fractal_index: 0,
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
                Button::new(format!("Reset {}", fractal.name)), // TODO change text with drop down menu name
            )
            .clicked()
        {
            *fractal = Self::default().fractals[self.fractal_index].clone();
        }

        let max_depth = max_depth_with_branches(
            MAX_PAINTED_LINE_COUNT,
            fractal.design_line_count,
            fractal.mirror,
            fractal.replace_line,
        );

        ui.label(format!("Painted line count: {}", self.line_count));
        ui.checkbox(&mut fractal.replace_line, "Replace parent with children");
        ui.checkbox(&mut fractal.mirror, "Mirror");
        ui.checkbox(&mut fractal.rainbow, "Rainbow");
        let iterator_count = &fractal.design_lines.len() - 1;
        ui.add(
            Slider::new(&mut fractal.design_line_count, 1..=iterator_count)
                .text("Design line count"),
        );
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
            ui.add(Slider::new(&mut fractal.start_line_width, 0.0..=7.0).text("Start line width"))
        };
        ui.add(Slider::new(&mut fractal.depth, 0..=max_depth).text("depth"));

        egui::reset_button(ui, self, "Full Reset"); // NOTE: will not looked disabled, because of self.line_count

        ui.add(egui::github_link_file!(
            "https://github.com/mjvandermeulen/egui-fractals/blob/main/",
            "Source code."
        ));
    }

    fn design(&mut self, ui: &Ui, painter: &Painter) -> Vec<VectoredDesignLine> {
        handle_keyboard_input(ui, self);

        let fractal = &mut self.fractals[self.fractal_index];
        let to_screen = emath::RectTransform::from_to(
            Rect::from_center_size(
                pos2(fractal.center.x, fractal.center.y),
                painter.clip_rect().square_proportions() / fractal.zoom,
            ),
            painter.clip_rect(),
        );
        let from_screen = to_screen.inverse();

        let rect = painter.clip_rect();
        let id = ui.make_persistent_id("design_painter_interaction");

        // Mouse input

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
            if self.dragged_line_end_point.is_none()
                && let Some(screenpos) = click_and_drag_response.interact_pointer_pos()
            {
                let local_pos = from_screen * screenpos;
                self.dragged_line_end_point = closest_handle(
                    local_pos,
                    &fractal.design_lines[..fractal.design_line_count + 1],
                    &fractal.lines_style,
                );
            }
            if let Some([line, end]) = self.dragged_line_end_point {
                let tuning_ratio = if self.fine_tune { 0.02 } else { 1.0 };
                let new_point = from_screen
                    * (to_screen * fractal.design_lines[line].line[end]
                        + tuning_ratio * click_and_drag_response.drag_delta());
                if fractal.lines_style == LinesStyle::Loop {
                    debug_assert_ne!(
                        end, 0,
                        "Loop style expects that the start point of a line can not be dragged"
                    );
                    fractal.design_lines[line].line[1] = new_point;
                    let next_line_index = (line + 1) % (fractal.design_line_count + 1);
                    fractal.design_lines[next_line_index].line[0] = new_point;
                } else if fractal.lines_style == LinesStyle::Tree {
                    fractal.design_lines[line].line[end] = new_point;
                    if line == 0 && end == 1 {
                        fractal
                            .design_lines
                            .iter_mut()
                            .skip(1)
                            .for_each(|d_line| d_line.line[0] = new_point);
                    }
                } else {
                    fractal.design_lines[line].line[end] = new_point;
                }
            }
        } else {
            self.dragged_line_end_point = None;
            if click_and_drag_response.double_clicked()
                && let Some(screen_pos) = ui.input(|i| i.pointer.hover_pos())
            {
                let pos = from_screen * screen_pos;
                if let Some(i) = closest_line(pos, &fractal.design_lines) {
                    fractal.design_lines[i].reversed = !fractal.design_lines[i].reversed;
                }
            }
        }

        design_lines_to_global_design_vectors(
            &fractal.design_lines[..fractal.design_line_count + 1],
            to_screen,
        )
    }

    fn paint_design(&self, painter: &Painter, design_vectors: &[VectoredDesignLine]) {
        let fractal = &self.fractals[self.fractal_index];
        design_vectors.iter().enumerate().for_each(|(i, vec)| {
            let (width, color) = if i == 0 {
                (fractal.start_line_width * 1.5, Color32::RED)
            } else {
                (fractal.start_line_width, Color32::ORANGE)
            };
            paint_directed_line_segment(painter, vec, width, color);
        });
    }

    fn paint_fractal(&mut self, painter: &Painter, vectored_design_lines: &[VectoredDesignLine]) {
        let fractal = &self.fractals[self.fractal_index];

        debug_assert!(
            fractal.depth
                <= max_depth_with_branches(
                    MAX_PAINTED_LINE_COUNT,
                    vectored_design_lines.len() - 1,
                    fractal.mirror,
                    fractal.replace_line
                ),
            "fractal.depth = {}, max_depth_with_branches(...) = {}",
            fractal.depth,
            max_depth_with_branches(
                MAX_PAINTED_LINE_COUNT,
                vectored_design_lines.len() - 1,
                fractal.mirror,
                fractal.replace_line
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
                if fractal.mirror {
                    line_transforms.push(LineTransform::from_design_vector(&base, *line, true));
                }
                line_transforms
            })
            .collect();
        let base_line_width = if fractal.replace_line {
            fractal.fixed_final_line_width
        } else {
            fractal.start_line_width
        };
        if !fractal.replace_line || fractal.depth == 0 {
            paint_line(
                [base.pos, base.pos + base.vec],
                line_color(0, fractal.rainbow),
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
        for depth in 1..fractal.depth + 1 {
            let color = line_color(depth, fractal.rainbow);

            if depth < fractal.depth {
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

                    if fractal.replace_line {
                        if depth == fractal.depth {
                            paint_line([paint_a, paint_b], color, fractal.fixed_final_line_width);
                        }
                    } else {
                        paint_line(
                            [paint_a, paint_b],
                            color,
                            (painted_node.vec.length() / base_length) * fractal.start_line_width,
                        );
                    }
                    if depth < fractal.depth {
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
        let fractal = &mut self.fractals[self.fractal_index];

        fractal.depth = fractal.depth.at_most(max_depth_with_branches(
            // TODO move this to end of design. Add it to paint_fractal as a dbg assert
            MAX_PAINTED_LINE_COUNT,
            fractal.design_line_count,
            fractal.mirror,
            fractal.replace_line,
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
