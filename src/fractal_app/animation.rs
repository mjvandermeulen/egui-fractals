use egui::{Pos2, emath::Rot2};

use super::structs_and_enums::VectoredLine;

use animation_tools::animation_progress_to_scale;

// TODO check if these need to be pub
pub mod animation_structs_and_enums;
pub mod animation_tools;

pub fn scale_and_rotate_vectored_lines_around_rotation_center(
    lines: &[VectoredLine],
    rotation_center: Pos2,
    cycle_angle: f32,
    cycle_scale: f32,
    progress: f32,
) -> Vec<VectoredLine> {
    let angle = progress * cycle_angle;
    let scale = animation_progress_to_scale(progress, cycle_scale);
    let rotation = Rot2::from_angle(angle);
    lines
        .iter()
        .map(|line| VectoredLine {
            pos: rotation_center + rotation * (line.pos - rotation_center) * scale,
            vec: rotation * line.vec * scale,
        })
        .collect()
}
