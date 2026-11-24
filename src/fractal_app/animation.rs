use egui::{Pos2, Vec2, emath::Rot2};

use super::structs_and_enums::PointVectorSegment;

// TODO check if these need to be pub
/// The rotation here is a pure non-scaling rotation.
/// This is easier to read and performance is not an issue here.
pub mod animation_helpers;
pub mod animation_structs_and_enums;
pub mod player_state;

#[must_use]
pub fn scale_rotate_and_translate_segments(
    segments: &[PointVectorSegment],
    rotation_center: Pos2,
    scale: f32,
    rotation: Rot2,
    translation: Option<Vec2>,
) -> Vec<PointVectorSegment> {
    let translation = translation.unwrap_or(Vec2::ZERO);
    segments
        .iter()
        .map(|segment| PointVectorSegment {
            start: rotation_center
                + rotation * (segment.start - rotation_center) * scale
                + translation,
            delta: rotation * segment.delta * scale,
        })
        .collect()
}
