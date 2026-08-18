use super::animation::animation_structs_and_enums::Animation;
use egui::{Pos2, Vec2, emath::RectTransform};

// Fractal struct

#[derive(PartialEq, Eq, Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub enum LinesStyle {
    Free,
    Tree,
    Loop,
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LineHandles {
    SingleHandle(usize),
    BothHandles, // or Both...
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct ReversibleLine {
    pub line: [Pos2; 2],
    pub reversed: bool,
}
#[derive(PartialEq, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Fractal {
    pub name: String,
    pub mirror: bool,
    pub rainbow: bool,
    pub design_lines: Vec<ReversibleLine>,
    pub replace_line: bool,
    pub lines_style: LinesStyle,
    pub zoom: f32,
    pub center: Pos2,
    pub initiator_width_length_ratio: f32,
    pub fixed_final_line_width: f32,
    pub depth: usize,
    pub animation: Animation,
}

// design structs

#[derive(Debug, Clone, Copy)]
pub struct VectoredLine {
    pub pos: Pos2,
    pub vec: Vec2,
}

impl VectoredLine {
    pub fn from_reversible_line(
        ReversibleLine { line, reversed }: ReversibleLine,
        to_screen: RectTransform,
    ) -> Self {
        let (start, end) = if reversed {
            (to_screen * line[1], to_screen * line[0])
        } else {
            (to_screen * line[0], to_screen * line[1])
        };

        let vec = end - start;
        Self { pos: start, vec }
    }
}

// paint_fractal structs

#[derive(Clone, Copy)]
// OPTI!!!: add length and add length factor to LineTransform,
//   then the lengthfactor to calculate width does not need multiple length calcs
//   (pythagoras sqrt is expensive)
pub struct Node {
    pub pos: Pos2,
    pub vec: Vec2,
}

#[derive(Clone, Copy)]
pub struct LineTransform {
    pub base_rot: egui::emath::Rot2,
    pub rot: egui::emath::Rot2,
}

impl LineTransform {
    pub fn from_design_vector(
        base: &VectoredLine,
        design_line: VectoredLine,
        mirrored: bool,
    ) -> Self {
        let base_to_dcl: Vec2 = design_line.pos - base.pos;
        let mirror_sign: f32 = if mirrored { -1.0 } else { 1.0 };
        Self {
            base_rot: base_to_dcl.length() / base.vec.length()
                * egui::emath::Rot2::from_angle(
                    mirror_sign * (base_to_dcl.angle() - base.vec.angle()),
                ),
            rot: design_line.vec.length() / base.vec.length()
                * egui::emath::Rot2::from_angle(
                    mirror_sign * (design_line.vec.angle() - base.vec.angle()),
                ),
        }
    }
}
