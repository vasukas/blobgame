use std::f32::consts::TAU;

/// Linear interpolation
pub fn lerp<T: std::ops::Add<Output = T> + std::ops::Mul<f32, Output = T>>(
    v0: T, v1: T, t: f32,
) -> T {
    v0 * (1. - t) + v1 * t // more precise than `v0 + t * (v1 - v0)`
}

/// Calculate `t` in `value = lerp(v0, v1, t)`
pub fn inverse_lerp<T: std::ops::Sub<Output = T> + std::ops::Div<Output = f32> + Copy>(
    value: T, v0: T, v1: T,
) -> f32 {
    (value - v0) / (v1 - v0)
}

//

pub trait FloatExtended {
    /// Same as `(self * TAU).sin() * 0.5 + 0.5`
    fn t_sin(self) -> Self;
}

impl FloatExtended for f32 {
    fn t_sin(self) -> Self {
        (self * TAU).sin() * 0.5 + 0.5
    }
}
