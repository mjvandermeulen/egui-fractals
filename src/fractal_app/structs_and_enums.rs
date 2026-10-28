use std::time::{Duration, Instant};

use super::animation::animation_structs_and_enums::Animation;
use egui::{Pos2, Vec2, emath::RectTransform};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum AnimationState {
    Playing {
        previously_elapsed_time: Duration,
        start_time: Instant,
    },
    PlayingReverse {
        previously_elapsed_time: Duration,
        start_time: Instant,
    },
    Paused {
        elapsed: Duration,
    },
    Stopped,
}

impl AnimationState {
    pub fn elapsed(&self) -> Duration {
        match *self {
            Self::Playing {
                previously_elapsed_time,
                start_time,
            } => previously_elapsed_time + start_time.elapsed(),
            Self::PlayingReverse {
                previously_elapsed_time,
                start_time,
            } => {
                // saturating_sub clamps at ZERO instead of panicking on underflow
                previously_elapsed_time.saturating_sub(start_time.elapsed())
            }
            Self::Paused { elapsed } => elapsed,
            Self::Stopped => Duration::ZERO,
        }
    }

    /// Call once per frame, before reading the time. Switches to Stopped
    /// when reverse playback reaches the beginning.
    pub fn update(&mut self) {
        if let Self::PlayingReverse {
            previously_elapsed_time,
            start_time,
        } = *self
            && start_time.elapsed() >= previously_elapsed_time
        {
            *self = Self::Stopped;
        }
    }

    pub fn is_playing(&self) -> bool {
        matches!(self, Self::Playing { .. } | Self::PlayingReverse { .. })
    }
}

// Fractal struct

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VectoredLine {
    pub pos: Pos2,
    pub vec: Vec2,
}

impl VectoredLine {
    #[must_use]
    pub fn from_reversible_line(
        ReversibleLine { line, reversed }: ReversibleLine,
        to_screen: RectTransform,
    ) -> Self {
        let (start, end) = if reversed {
            (to_screen * line[1], to_screen * line[0])
        } else {
            (to_screen * line[0], to_screen * line[1])
        };

        let vec = end - start;
        Self { pos: start, vec }
    }
}

// paint_fractal structs

#[derive(Clone, Copy)]
// OPTI!!!: add length and add length factor to LineTransform,
//   then the lengthfactor to calculate width does not need multiple length calcs
//   (pythagoras sqrt is expensive)
// only used in parallel_ and paint_fractal_helpers. If parallel gets fully removed,
// then move this to paint_fractal_helpers and remove all `pub`

// TODO move this to paint fractal helpers and remove 'pub'
pub struct Branch {
    pub parent_node: Pos2,
    pub edge: Vec2,
}

impl Branch {
    #[must_use]
    pub fn from_vectored_line(VectoredLine { pos, vec }: VectoredLine) -> Self {
        Self {
            parent_node: pos,
            edge: vec,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LineTransform {
    pub base_rot: egui::emath::Rot2,
    pub rot: egui::emath::Rot2,
}

impl LineTransform {
    #[must_use]
    pub fn from_design_vector(
        base: &VectoredLine,
        design_line: VectoredLine,
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
