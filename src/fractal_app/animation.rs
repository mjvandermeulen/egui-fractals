use egui::{Pos2, emath::Rot2};

use super::structs_and_enums::PointVectorSegment;

// TODO check if these need to be pub
pub mod animation_helpers;
pub mod animation_structs_and_enums;
pub mod player_state;

pub fn scale_and_rotate_vectored_lines_around_rotation_center(
    lines: &[PointVectorSegment],
    rotation_center: Pos2,
    scale: f32,
    rotation: Rot2,
) -> Vec<PointVectorSegment> {
    lines
        .iter()
        .map(|line| PointVectorSegment {
            start: rotation_center + rotation * (line.start - rotation_center) * scale,
            delta: rotation * line.delta * scale,
        })
        .collect()
}
