use std::f32::consts::{PI, TAU};

/// Linear interpolation
pub fn lerp<T: std::ops::Add<Output = T> + std::ops::Mul<f32, Output = T>>(
    v0: T, v1: T, t: f32,
) -> T {
    v0 * (1. - t) + v1 * t // more precise than `v0 + t * (v1 - v0)`
}

/// Calculate `t` in `value = lerp(v0, v1, t)` where lerp is linear interpolation
pub fn inverse_lerp<T: std::ops::Sub<Output = T> + std::ops::Div<Output = f32> + Copy>(
    value: T, v0: T, v1: T,
) -> f32 {
    (value - v0) / (v1 - v0)
}

/// Smallest difference between angles, output range is \[-π; +π\]
/// 
/// Adding result of this function to `current` will make it equal to `target`.
pub fn angle_delta(target: f32, current: f32) -> f32 {
    // Source: https://stackoverflow.com/a/28037434
    let diff = (target - current + PI) % TAU - PI;
    if diff < -PI {
        diff + TAU
    } else {
        diff
    }
}

//

pub trait FloatExtended {
    /// Same as `(self * TAU).sin() * 0.5 + 0.5`.
    /// 
    /// Useful for transforming linear `[0; 1]` into sinusoidal `[0; 1]` for interpolation
    fn t_sin(self) -> Self;
}

impl FloatExtended for f32 {
    fn t_sin(self) -> Self {
        (self * TAU).sin() * 0.5 + 0.5
    }
}
