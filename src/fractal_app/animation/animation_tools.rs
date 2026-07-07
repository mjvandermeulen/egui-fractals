use std::time::Instant;

pub fn animation_progress(start_time: Instant, animation_length: f32) -> f32 {
    let elapsed_time = start_time.elapsed().as_secs_f32();
    elapsed_time / animation_length
}
