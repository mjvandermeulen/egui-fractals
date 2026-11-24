use super::animation::animation_structs_and_enums::Animation;
use egui::{Pos2, Vec2, emath::RectTransform};
use serde::{Deserialize, Serialize};
pub struct NearestPoint {
    target: Pos2,
    best_distance_sq: f32, // no need for an Option, only checked if point is Some
    pub point: Option<Pos2>,
}

impl NearestPoint {
    #[expect(dead_code)]
    pub fn new(target: Pos2) -> Self {
        Self {
            target,
            best_distance_sq: f32::INFINITY, // never used before overwritten.. could be any number
            point: None,
        }
    }

    #[expect(dead_code)]
    pub fn observe(&mut self, candidate: Pos2) {
        let d = self.target.distance_sq(candidate);
        if self.point.is_none() || self.best_distance_sq > d {
            self.best_distance_sq = d;
            self.point = Some(candidate);
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Deserialize, Serialize)]
pub enum LinesStyle {
    Free,
    Tree,
    Loop,
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LineHandles {
    SingleHandle(usize), // 0: start handle, 1: end handle
    BothHandles,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Deserialize, Serialize)]
pub struct ReversibleLine {
    pub line: [Pos2; 2],
    pub reversed: bool,
}
#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
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

/// Line segment, determined by:
/// - start: Start point.
/// - delta: The vector that maps the start to the endpoint.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PointVectorSegment {
    pub start: Pos2,
    pub delta: Vec2,
}

impl PointVectorSegment {
    #[must_use]
    pub fn from_reversible_line(
        ReversibleLine { line, reversed }: ReversibleLine,
        to_screen: RectTransform,
    ) -> Self {
        let (a, b) = (line[0], line[1]);
        let (start, end) = if reversed { (b, a) } else { (a, b) };
        let (start, end) = (to_screen * start, to_screen * end);
        Self {
            start,
            delta: end - start,
        }
    }
}

// paint_fractal structs

#[derive(Debug, Clone, Copy)]
// OPTI!!!: add length and add length factor to LineTransform,
//   then the lengthfactor to calculate width does not need multiple length calcs
//   (pythagoras sqrt is expensive)
// only used in parallel_ and paint_fractal_helpers. If parallel gets fully removed,
// then move this to paint_fractal_helpers and remove all `pub`

// TODO move this to paint fractal helpers and remove 'pub'
//
// NOTE: This is internally identical to PointVectorSegement, but needed in a different part of the painting.
// Mixing them up would cause an error. This is now prevented. The compiler won't allow using them interchangebly.
pub struct Branch {
    pub start: Pos2,
    pub vec: Vec2,
}

impl Branch {
    #[must_use]
    pub fn from_point_vector_segment(
        PointVectorSegment { start, delta: vec }: PointVectorSegment,
    ) -> Self {
        Self { start, vec }
    }

    /// Constructs a child branch by performing the given `BranchTransform` on the current branch.
    #[must_use]
    pub fn child(&self, transformation: BranchTransformation) -> Self {
        let start = self.start + transformation.base_rot_scale * self.vec;
        let vec: Vec2 = transformation.rot * self.vec;
        Self { start, vec }
    }
    #[must_use]
    pub fn line(&self) -> [Pos2; 2] {
        [self.start, self.start + self.vec]
    }
    #[must_use]
    pub fn length(&self) -> f32 {
        self.vec.length()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BranchTransformation {
    pub base_rot_scale: egui::emath::Rot2,
    pub rot: egui::emath::Rot2,
}

impl BranchTransformation {
    #[must_use]
    pub fn from_design_vector(
        base: &PointVectorSegment,
        design_line: PointVectorSegment,
        mirrored: bool,
    ) -> Self {
        let base_to_dcl: Vec2 = design_line.start - base.start;
        let mirror_sign: f32 = if mirrored { -1.0 } else { 1.0 };
        Self {
            base_rot_scale: base_to_dcl.length() / base.delta.length()
                * egui::emath::Rot2::from_angle(
                    mirror_sign * (base_to_dcl.angle() - base.delta.angle()),
                ),
            rot: design_line.delta.length() / base.delta.length()
                * egui::emath::Rot2::from_angle(
                    mirror_sign * (design_line.delta.angle() - base.delta.angle()),
                ),
        }
    }
}
