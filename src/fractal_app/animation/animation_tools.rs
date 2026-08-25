use std::time::Instant;

use egui::Pos2;

pub fn animation_progress(
    start_time: Instant,
    animation_length: f32,
    repeat_cycle: Option<usize>,
) -> f32 {
    let elapsed_time = start_time.elapsed().as_secs_f32();

    match repeat_cycle {
        Some(rc) => {
            let progress_at_cycle_start = rc as f32;
            let progress_in_cycle = (elapsed_time % animation_length) / animation_length;
            // log::info!(
            //     "progress_at_cycle_start: {progress_at_cycle_start}. progress_in_cycle: {progress_in_cycle}"
            // );
            progress_at_cycle_start + progress_in_cycle
        }
        None => elapsed_time / animation_length,
    }
}

pub fn animation_progress_to_scale(progress: f32, cycle_scale: f32) -> f32 {
    cycle_scale.powf(progress)
}

pub fn find_animation_rotation_center(
    b: Pos2,
    c: Pos2,
    angle_rad: f32,
    ratio_ac_ab: f32,
) -> Option<Pos2> {
    // 1. Compute the complex multiplier m = r * e^(i * alpha)
    let (sin_a, cos_a) = angle_rad.sin_cos();
    let m_re = ratio_ac_ab * cos_a;
    let m_im = ratio_ac_ab * sin_a;

    // 2. Compute the denominator (m - 1)
    let den_re = m_re - 1.0;
    let den_im = m_im;

    let den_mag_sq = den_re * den_re + den_im * den_im;
    if den_mag_sq < 1e-6 {
        return None; // Handles collinear/undefined cases where m ≈ 1
    }

    // 3. Compute the numerator (m * B - C)
    let mb_re = m_re * b.x - m_im * b.y;
    let mb_im = m_re * b.y + m_im * b.x;

    let num_re = mb_re - c.x;
    let num_im = mb_im - c.y;

    // 4. Complex division: Num / Den
    let a_x = (num_re * den_re + num_im * den_im) / den_mag_sq;
    let a_y = (num_im * den_re - num_re * den_im) / den_mag_sq;

    Some(Pos2::new(a_x, a_y))
}
