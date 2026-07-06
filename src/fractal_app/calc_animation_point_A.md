Got it. Let’s adjust the orientation to match your exact interpretation: the angle $\alpha$ is measured at point $A$, sweeping **from $B$ to $C$**, and our ratio is still $r = \frac{AB}{AC}$.

### Updated Geometric Derivation

Using complex numbers where $A$, $B$, and $C$ represent our points:

- The vector from $A$ to $B$ is $B - A$.
- The vector from $A$ to $C$ is $C - A$.

Because the angle $\alpha$ sweeps from $B$ to $C$, the transformation rotates the vector $(B - A)$ into the direction of $(C - A)$, while scaling it by the inverse ratio $\frac{1}{r} = \frac{AC}{AB}$.

$$\frac{C - A}{B - A} = \frac{1}{r} \cdot e^{i\alpha}$$

To make the algebraic steps cleaner, let’s define $m = \frac{1}{r} \cdot e^{i\alpha} = \frac{1}{r}(\cos\alpha + i\sin\alpha)$. Now we solve for $A$:

$$C - A = m(B - A)$$

$$C - A = mB - mA$$

$$mA - A = mB - C$$

$$A(m - 1) = mB - C$$

$$A = \frac{mB - C}{m - 1}$$

---

### Rust Implementation with `egui::Pos2`

Here is the updated implementation reflecting the $B \to C$ angle direction and ratio handling:

```rust
use egui::Pos2;

fn find_point_a_corrected(b: Pos2, c: Pos2, angle_rad: f32, ratio_ab_ac: f32) -> Option<Pos2> {
    // Avoid division by zero if the ratio is zero
    if ratio_ab_ac.abs() < 1e-6 {
        return None;
    }

    // 1. Compute inverse ratio for the B -> C scaling factor
    let inv_r = 1.0 / ratio_ab_ac;

    // 2. Compute the complex multiplier m = (1/r) * e^(i * alpha)
    let (sin_a, cos_a) = angle_rad.sin_cos();
    let m_re = inv_r * cos_a;
    let m_im = inv_r * sin_a;

    // 3. Compute the denominator (m - 1)
    let den_re = m_re - 1.0;
    let den_im = m_im;

    let den_mag_sq = den_re * den_re + den_im * den_im;
    if den_mag_sq < 1e-6 {
        return None; // Handles collinear/undefined edge cases
    }

    // 4. Compute the numerator (m * B - C)
    let mb_re = m_re * b.x - m_im * b.y;
    let mb_im = m_re * b.y + m_im * b.x;

    let num_re = mb_re - c.x;
    let num_im = mb_im - c.y;

    // 5. Complex division: Num / Den
    let a_x = (num_re * den_re + num_im * den_im) / den_mag_sq;
    let a_y = (num_im * den_re - num_re * den_im) / den_mag_sq;

    Some(Pos2::new(a_x, a_y))
}

```
