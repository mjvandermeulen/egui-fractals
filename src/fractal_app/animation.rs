use egui::{Pos2, emath::Rot2};

use super::structs_and_enums::VectoredLine;

// TODO check if these need to be pub
pub mod animation_structs_and_enums;
pub mod animation_tools;

pub fn design_to_frame_scale_and_rotation(
    cycle_angle: f32,
    cycle_scale: f32,
    progress: f32,
    r_cycle: usize, // the cycle that repeats
) -> (f32, Rot2) {
    if progress < r_cycle as f32 {
        (cycle_scale.powf(progress), Rot2::from_angle(0.0))
    } else {
        (
            cycle_scale.powf(r_cycle as f32 + progress.fract()),
            Rot2::from_angle((progress.floor() - r_cycle as f32) * -cycle_angle),
        )
    }
}

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
