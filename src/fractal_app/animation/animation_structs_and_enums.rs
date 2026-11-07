#[expect(unused_imports)]
use std::{f32::consts::LN_2, time::Duration};

use crate::structs_and_enums::PlayerState;

#[derive(PartialEq, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Animation {
    // TODO: usize: zoom focus branch.
    // next 2 lines: remove the pubs, for now needed for fractals.rs
    pub ln_scale: f64,            // the state; negative = zoomed out past the start
    pub ln_scale_per_second: f32, // private, use the setter
    pub animation_loop_cycle: usize, // the cycle that can be looped to give illusion of infinite animation.
    pub zoom_child_to_parent: bool, // true: scale and rotate child branch to match parent. false: zoom in without rotation.
    pub player_state: PlayerState,
}

impl Animation {
    pub fn new(ln_scale_per_second: f32) -> Self {
        Self {
            ln_scale: 0.0,
            ln_scale_per_second,
            animation_loop_cycle: 0,
            zoom_child_to_parent: true,
            player_state: PlayerState::Stopped,
        }
    }

    /// Call once per frame with `ctx.input(|i| i.stable_dt)`.
    pub fn update(&mut self, dt: f32) {
        // Clamp so a stalled frame (window drag, background tab) doesn't cause a jump.
        let dt = f64::from(dt.min(0.1));
        let step = f64::from(self.ln_scale_per_second) * dt;

        match self.player_state {
            PlayerState::Playing => self.ln_scale += step,
            PlayerState::PlayingReverse => {
                self.ln_scale -= step;
                if self.ln_scale <= 0.0 {
                    self.ln_scale = 0.0;
                    self.player_state = PlayerState::Stopped;
                }
            }
            PlayerState::PlayingReverseInfinite => self.ln_scale -= step,
            PlayerState::Paused | PlayerState::Stopped => {}
        }
    }

    pub fn is_playing(&self) -> bool {
        matches!(
            self.player_state,
            PlayerState::Playing
                | PlayerState::PlayingReverse
                | PlayerState::PlayingReverseInfinite
        )
    }

    pub fn is_stopped(&self) -> bool {
        self.player_state == PlayerState::Stopped
    }

    pub fn current_ln_scale(&self) -> f64 {
        self.ln_scale
    }

    pub fn ln_scale_per_second(&self) -> f32 {
        self.ln_scale_per_second
    }

    /// No rebasing needed: the rate only affects future frames.
    /// Rejects non-finite or zero rates.
    pub fn set_ln_scale_per_second(&mut self, rate: f32) {
        if rate.is_finite() && rate != 0.0 {
            self.ln_scale_per_second = rate;
        }
    }

    pub fn play(&mut self) {
        self.player_state = PlayerState::Playing;
    }

    pub fn play_reverse(&mut self) {
        // Nothing to rewind at the start.
        if self.ln_scale > 0.0 {
            self.player_state = PlayerState::PlayingReverse;
        }
    }

    pub fn play_reverse_infinite(&mut self) {
        self.player_state = PlayerState::PlayingReverseInfinite;
    }

    pub fn pause(&mut self) {
        if self.is_playing() {
            self.player_state = PlayerState::Paused;
        }
    }

    pub fn stop(&mut self) {
        self.ln_scale = 0.0;
        self.player_state = PlayerState::Stopped;
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
// All in units of 1 cycle
pub struct AnimationProgress {
    pub elapsed: f64, // elapsed cycles, including the fractional part of the current cycle.
    pub loop_repetitions: usize, // how many times the loop_cycle has been completed.
    pub elapsed_wrapped: f32, // If loop_cycle reached, this is loop_cycle plus phase. Otherwise it's elapsed.
    pub phase: f32,           // phase of the current cycle, in [0, 1)
}

impl AnimationProgress {
    pub fn new_from_ln_scale(ln_scale: f64, cycle_scale: f64, animation: &Animation) -> Self {
        let elapsed_cycles = ln_scale / cycle_scale.ln(); // convert ln_scale to elapsed_cycles
        let phase: f32 = elapsed_cycles.fract() as f32;
        let loop_cycle = animation.animation_loop_cycle as f64;
        Self {
            elapsed: elapsed_cycles,
            loop_repetitions: if elapsed_cycles < loop_cycle {
                0
            } else {
                (elapsed_cycles.floor() - loop_cycle) as usize
            },
            elapsed_wrapped: if elapsed_cycles < loop_cycle {
                elapsed_cycles as f32
            } else {
                loop_cycle as f32 + phase
            },
            phase,
        }
    }

    pub fn looping_started(&self) -> bool {
        self.elapsed > self.elapsed_wrapped as f64
    }
}
