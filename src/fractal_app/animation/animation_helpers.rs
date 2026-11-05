use egui::Pos2;

#[expect(clippy::similar_names)]
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
