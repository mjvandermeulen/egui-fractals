use std::time::Duration;

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub struct Animation {
    // TODO: usize: zoom focus branch.
    pub zooms_per_minute: f32, // NOTE: it could be nice if this is not pub, and only has a setter.
    pub animation_loop_cycle: usize,
    //   the cycle that can be looped to give illusion of infinite animation.
    //   first cycle = 0. (progress.floor() matches the cycle)
    //   e.g.: 3: loop the 4th cycle
}

impl Animation {
    pub fn ln_scale_per_second(&self) -> f32 {
        self.zooms_per_minute.ln() / 60.0
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
    pub fn new_from_elapsed_time(
        elapsed_time: Duration,
        cycle_scale: f32,
        animation: &Animation,
    ) -> Self {
        let seconds_per_cycle = cycle_scale.ln() / animation.ln_scale_per_second();

        let elapsed_cycles = elapsed_time.as_secs_f32() / seconds_per_cycle;
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
                loop_cycle + phase
            } else {
                elapsed_cycles
            },
            phase,
        }
    }
}
