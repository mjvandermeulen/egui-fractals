use egui::{Pos2, emath::Rot2};

use super::structs_and_enums::VectoredLine;

// TODO check if these need to be pub
pub mod animation_structs_and_enums;
pub mod animation_tools;

pub fn design_to_frame_scale_rotation_and_color_offset(
    cycle_angle: f32,
    cycle_scale: f32,
    progress: f32,
    r_cycle: usize, // the cycle that repeats
) -> (f32, Rot2, usize) {
    if progress < r_cycle as f32 {
        (
            cycle_scale.powf(progress), // scale
            Rot2::from_angle(0.0),      // rotation
            0,                          // color_offset
        )
    } else {
        let repeated_cycles = progress.floor() - r_cycle as f32;
        (
            cycle_scale.powf(r_cycle as f32 + progress.fract()), // scale
            Rot2::from_angle(repeated_cycles * -cycle_angle),    // rotation
            repeated_cycles as usize,                            // color_offset
        )
    }
}

pub fn scale_and_rotate_vectored_lines_around_rotation_center(
    lines: Vec<VectoredLine>,
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
