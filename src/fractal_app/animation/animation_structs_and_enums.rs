#[expect(unused_imports)]
use std::{f32::consts::LN_2, time::Duration};

use egui::NumExt as _;

use crate::fractal_app::animation::player_state::{PlayerState, ReverseMode};

//
// the boundary between left and right:        scale = loop_cycle_scale
//                                                   |
//             ┌────────────────────────────────────┐┌────────────────────────────────────┐
//             │ 0.0 <= scale < loop_cycle_scale    ││ scale >= loop_cycle_scale          │
//             │◄─ play reverse ────────────────────││◄─ play reverse ────────────────────│
//             │── play ───────────────────────────►││── play ───────────────────────────►│
//             └────────────────────────────────────┘│                                    │
//                                                   │------------------------------------│
// ┌────────────────────────────────────────────────┐│                                    │
// │ scale < loop_cycle_scale                       ││ scale >= loop_cycle_scale          │
// │◄─ play infinite reverse ───────────────────────││◄─ play infinite reverse ───────────│
// │── play ───────────────────────────────────────►││── play ───────────────────────────►│
// └────────────────────────────────────────────────┘└────────────────────────────────────┘
//
//
#[derive(PartialEq, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Animation {
    // IDEA: usize: zoom focus branch.
    // next 2 lines: remove the pubs, for now needed for fractals.rs
    // OR BETTER: remove the getter and setter. TODO!!!!
    pub ln_scale: f64,            // the state; negative = zoomed out past the start
    pub ln_scale_per_second: f32, // private, use the setter
    pub animation_loop_cycle: usize, // the cycle that can be looped to give illusion of infinite animation.
    pub zoom_child_to_parent: bool, // true: scale and rotate child branch to match parent. false: zoom in without rotation.
    pub player_state: PlayerState,
    pub loop_scale_reached: bool, // ugly: this is a calculated value. MAYBE TODO!!!! change to current_loop_scale?
}

impl Animation {
    // Rejects non-finite or zero rates.
    pub fn set_ln_scale_per_second(&mut self, rate: f32) {
        if rate.is_finite() && rate != 0.0 {
            self.ln_scale_per_second = rate;
        }
    }

    /// Call once per frame with `ctx.input(|i| i.stable_dt)`.
    pub fn update(&mut self, dt: f32) {
        // Clamp so a stalled frame (window drag, background tab) doesn't cause a jump.
        let dt = f64::from(dt.at_most(0.1));
        let step = f64::from(self.ln_scale_per_second) * dt;

        match self.player_state {
            PlayerState::PlayingForward(_) => self.ln_scale += step,
            PlayerState::PlayingReverse => {
                self.ln_scale -= step;
                if self.ln_scale <= 0.0 {
                    self.stop();
                }
            }
            PlayerState::PlayingReverseInfinite => {
                self.ln_scale -= step;
            }
            PlayerState::Paused(_) => {}
        }
    }

    pub fn reversing_mode(&self) -> ReverseMode {
        match self.player_state {
            PlayerState::Paused(_) if self.ln_scale == 0.0 => ReverseMode::Reverse,
            PlayerState::Paused(reversing_mode) | PlayerState::PlayingForward(reversing_mode) => {
                reversing_mode
            }
            PlayerState::PlayingReverseInfinite => ReverseMode::ReverseInfinite,
            PlayerState::PlayingReverse => ReverseMode::Reverse,
        }
    }

    // --- player state getters ---
    /// Returns `true` if playing in any direction.
    pub fn not_paused_or_stopped(&self) -> bool {
        !matches!(self.player_state, PlayerState::Paused(_))
    }
    pub fn is_playing_forward(&self) -> bool {
        matches!(self.player_state, PlayerState::PlayingForward(_))
    }
    #[expect(unused)] // maybe needed later, to highlight button
    pub fn is_paused(&self) -> bool {
        matches!(self.player_state, PlayerState::Paused(_) if self.ln_scale != 0.0)
    }
    pub fn is_stopped(&self) -> bool {
        matches!(self.player_state, PlayerState::Paused(_) if self.ln_scale == 0.0)
    }
    pub fn ln_scale_per_second(&self) -> f32 {
        self.ln_scale_per_second
    }

    // --- handle button input ---
    pub fn play(&mut self) {
        self.player_state = PlayerState::PlayingForward(self.reversing_mode());
    }
    pub fn play_reverse(&mut self) {
        // Nothing to rewind at the start.
        if self.ln_scale > 0.0 {
            self.player_state = PlayerState::PlayingReverse;
        } else {
            log::warn!("play_reverse should not be reachable when ln_scale <= 0.0");
            self.stop();
        }
    }
    pub fn play_reverse_infinite(&mut self) {
        self.player_state = PlayerState::PlayingReverseInfinite;
    }
    pub fn pause(&mut self) {
        self.player_state = PlayerState::Paused(self.reversing_mode());
    }
    pub fn stop(&mut self) {
        self.ln_scale = 0.0;
        self.player_state = PlayerState::Paused(ReverseMode::Reverse);
    }

    // --- button enabled helpers ---
    pub fn reverse_infinite_is_enabled(&self) -> bool {
        self.player_state != PlayerState::PlayingReverseInfinite
            && (self.reversing_mode() != ReverseMode::Reverse || self.loop_scale_reached)
    }
    pub fn reverse_is_enabled(&self) -> bool {
        self.player_state != PlayerState::PlayingReverse
            && (self.reversing_mode() != ReverseMode::ReverseInfinite || self.loop_scale_reached)
            && self.ln_scale > 0.0
    }
    pub fn pause_is_enabled(&self) -> bool {
        self.not_paused_or_stopped()
    }
    pub fn play_forward_is_enabled(&self) -> bool {
        !self.is_playing_forward()
    }
    pub fn stop_is_enabled(&self) -> bool {
        !self.is_stopped()
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
// All in units of 1 cycle
// TODO!!!!! this needs to be run every frame, in case the user changes the cycle_scale
//   and the buttons need to be updated.
//   make progress part of the Animation? combine with Animation::update...
pub struct AnimationProgress {
    pub elapsed_cycles: f64, // elapsed cycles, including the fractional part of the current cycle. Claude recommends f64 for this, because it can be large and we want to avoid rounding errors. The fractional part is the phase of the current cycle.
    pub loop_repetitions: i128, // how many times the loop_cycle has been completed. // could be negative, scaling down.
    pub elapsed_wrapped: f32, // If loop_cycle reached, this is loop_cycle plus phase. Otherwise it's elapsed.
}

impl AnimationProgress {
    pub fn new_from_cycle_scale(cycle_scale: f64, animation: &mut Animation) -> Self {
        let elapsed_cycles = animation.ln_scale / cycle_scale.ln(); // convert ln_scale to elapsed_cycles
        let phase: f32 = elapsed_cycles.rem_euclid(1.0) as f32; // NOTE: -4.3.fract = -0.3 which is wrong.
        let loop_cycle = animation.animation_loop_cycle as f64;

        animation.loop_scale_reached = elapsed_cycles > loop_cycle;

        if !animation.loop_scale_reached && animation.reversing_mode() == ReverseMode::Reverse {
            Self {
                elapsed_cycles,
                loop_repetitions: 0,
                elapsed_wrapped: elapsed_cycles as f32,
            }
        } else {
            Self {
                elapsed_cycles,
                loop_repetitions: (elapsed_cycles.floor() - loop_cycle) as i128,
                elapsed_wrapped: loop_cycle as f32 + phase,
            }
        }
    }

    pub fn phase(&self) -> f32 {
        self.elapsed_cycles.rem_euclid(1.0) as f32
    }

    pub fn looping_started(&self) -> bool {
        self.elapsed_cycles > self.elapsed_wrapped as f64
    }
}
