use egui::{Pos2, Vec2};

#[derive(Clone, Copy)]
pub struct Node {
    pub pos: Pos2,
    pub dir: Vec2,
}

#[derive(Clone, Copy)]
pub struct LineTransform {
    pub base_rot: egui::emath::Rot2,
    pub rot: egui::emath::Rot2,
}

impl LineTransform {
    pub fn from_design_vector(
        base: &super::design_structs_and_helpers::VectoredDesignLine,
        design_line: super::design_structs_and_helpers::VectoredDesignLine,
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
