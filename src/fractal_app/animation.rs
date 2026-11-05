use egui::{Pos2, emath::Rot2};

use super::structs_and_enums::VectoredLine;

// TODO check if these need to be pub
pub mod animation_helpers;
pub mod animation_structs_and_enums;

pub fn scale_and_rotate_vectored_lines_around_rotation_center(
    lines: &[VectoredLine],
    rotation_center: Pos2,
    scale: f32,
    rotation: Rot2,
) -> Vec<VectoredLine> {
    lines
        .iter()
        .map(|line| VectoredLine {
            pos: rotation_center + rotation * (line.pos - rotation_center) * scale,
            vec: rotation * line.vec * scale,
        })
        .collect()
}
