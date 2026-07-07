use egui::{Pos2, Vec2, emath::Rot2};

use crate::fractal_app::structs_and_enums::DesignLine;

pub mod animation_structs_and_enums;
pub mod animation_tools;

// move to tools (rename to helpers.rs)
// to get the Rot2, for all points:
// let rotation = Rot2::from_angle(angle_radians);
pub fn rotate_point_around_center(point: Pos2, center: Pos2, rotation: Rot2) -> Pos2 {
    let point_vec: Vec2 = point - center;
    let rotated_vec: Vec2 = rotation * point_vec;
    let rotated_point: Pos2 = center + rotated_vec;
    rotated_point
}

pub fn rotate_design_lines(
    dls: &[DesignLine],
    rotation_center: Pos2,
    angle: f32,
) -> Vec<DesignLine> {
    let rotation = Rot2::from_angle(angle);
    dls.iter()
        .map(|dl| {
            let new_line = [
                rotate_point_around_center(dl.line[0], rotation_center, rotation),
                rotate_point_around_center(dl.line[1], rotation_center, rotation),
            ];
            DesignLine {
                line: new_line,
                reversed: dl.reversed,
            }
        })
        .collect()
}
