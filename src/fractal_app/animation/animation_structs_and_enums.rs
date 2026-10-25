#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub struct Animation {
    // TODO: usize: zoom focus branch.
    pub cycle_length: f32,
    pub animation_repetition_cycle: usize,
    //   the cycle that can be repeated to make a continuous animation.
    //   first cycle = 0. (progress.floor() matches the cycle)
    //   e.g.: 3: repeat the 4th cycle
}
