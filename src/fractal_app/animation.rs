use egui::{Pos2, Vec2, emath::Rot2};

use super::structs_and_enums::VectoredLine;

use animation_tools::animation_progress_to_scale;

// TODO check if these need to be pub
pub mod animation_structs_and_enums;
pub mod animation_tools;

// move to tools (rename to helpers.rs)
// to get the Rot2, for all points:
// let rotation = Rot2::from_angle(angle_radians);
// TODO!!!!Rot2 can have a scale factor
pub fn scale_and_rotate_point_around_center(point: Pos2, center: Pos2, rotation: Rot2) -> Pos2 {
    let point_vec: Vec2 = point - center;
    let rotated_vec: Vec2 = rotation * point_vec;
    let rotated_point: Pos2 = center + rotated_vec;
    rotated_point
}

pub fn scale_and_rotate_vectored_lines(
    lines: &[VectoredLine],
    rotation_center: Pos2,
    cycle_angle: f32,
    cycle_scale: f32,
    progress: f32,
) -> Vec<VectoredLine> {
    let angle = progress * cycle_angle;
    let scale = animation_progress_to_scale(progress, cycle_scale);
    let rotation = scale * Rot2::from_angle(angle);
    lines
        .iter()
        .map(|line| VectoredLine {
            pos: scale_and_rotate_point_around_center(line.pos, rotation_center, rotation),
            vec: rotation * line.vec,
        })
        .collect()
}
