use std::time::Instant;
#[expect(unused_imports)]
use std::{f32::consts::LN_2, time::Duration};

use crate::structs_and_enums::PlayerState;

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub struct Animation {
    // TODO: usize: zoom focus branch.
    pub ln_scale_per_second: f32, // NOTE: it could be nice if this is not pub, and only has a setter.
    pub animation_loop_cycle: usize, // the cycle that can be looped to give illusion of infinite animation.
    pub zoom_child_to_parent: bool, // if true, the animation will scale and rotate the child branch to match the parent branch. If false, it will zoom in without rotation.

    // TODO!!!!!!!!!: MOVE AnimationState in here!
    // - so it can be serialized/deserialized with the fractal settings.
    // - so it can be used in the animation system without needing to pass around a separate AnimationState.
    // - so when changing the ln_scale_per_second, the animation state can be updated to the new scale/sec.
    pub player_state: PlayerState,
}

impl Animation {
    pub fn current_ln_scale(&self) -> f32 {
        match self.player_state {
            PlayerState::Playing {
                ln_scale_at_time_stamp: ln_scale,
                time_stamp,
            } => ln_scale + time_stamp.elapsed().as_secs_f32() * self.ln_scale_per_second,
            PlayerState::PlayingReverse {
                ln_scale_at_time_stamp: ln_scale,
                time_stamp,
            } => {
                // saturating_sub clamps at ZERO instead of panicking on underflow
                ln_scale - time_stamp.elapsed().as_secs_f32() * self.ln_scale_per_second
            }
            PlayerState::Paused { ln_scale } => ln_scale,
            PlayerState::Stopped => 0.0,
        }
    }

    pub fn set_current_ln_scale(&mut self) {
        match self.player_state {
            PlayerState::Playing { .. } | PlayerState::PlayingReverse { .. } => {
                self.player_state = PlayerState::Playing {
                    ln_scale_at_time_stamp: self.current_ln_scale(),
                    time_stamp: Instant::now(),
                }
            }
            // NOTE: Paused and Stopped states do not need to update the ln_scale, as they are not changing over time.
            _ => {}
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
// All in units of 1 cycle
pub struct AnimationProgress {
    pub elapsed: f32, // elapsed cycles, including the fractional part of the current cycle.
    pub loop_repetitions: usize, // how many times the loop_cycle has been completed.
    pub elapsed_wrapped: f32, // If loop_cycle reached, this is loop_cycle plus phase. Otherwise it's elapsed.
    pub phase: f32,           // phase of the current cycle, in [0, 1)
}

impl AnimationProgress {
    pub fn new_from_ln_scale(ln_scale: f32, cycle_scale: f32, animation: &Animation) -> Self {
        let elapsed_cycles = ln_scale / cycle_scale.ln(); // convert ln_scale to elapsed_cycles
        let phase = elapsed_cycles.fract();
        let loop_cycle = animation.animation_loop_cycle as f32;
        Self {
            elapsed: elapsed_cycles,
            loop_repetitions: if elapsed_cycles < loop_cycle {
                0
            } else {
                (elapsed_cycles.floor() - loop_cycle) as usize
            },
            elapsed_wrapped: if elapsed_cycles < loop_cycle {
                elapsed_cycles
            } else {
                loop_cycle + phase
            },
            phase,
        }
    }

    pub fn looping_started(&self) -> bool {
        self.elapsed > self.elapsed_wrapped
    }
}
