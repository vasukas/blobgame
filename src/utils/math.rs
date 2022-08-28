use std::{
    f32::consts::{PI, TAU},
    time::Duration,
};

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

/// Magic
#[derive(Default)]
pub struct InterpolatedValue<T> {
    pub length: Duration,
    pub value: T,
    pub next_value: Option<(T, Option<Duration>)>,
}

impl<T: std::ops::Add<Output = T> + std::ops::Mul<f32, Output = T> + Clone + Copy + PartialEq>
    InterpolatedValue<T>
{
    pub fn set_next(&mut self, value: T) {
        if value != self.value && self.next_value.map(|v| v.0 != value).unwrap_or(true) {
            self.next_value = Some((value, None))
        }
    }
    pub fn update(&mut self, time: Duration) -> T {
        match self.next_value.as_mut() {
            Some((next, since)) => {
                let t = time
                    .checked_sub(*since.get_or_insert(time))
                    .unwrap_or_default()
                    .as_secs_f32()
                    / self.length.as_secs_f32();
                if t >= 1. {
                    self.value = *next;
                    self.next_value = None;
                    self.value
                } else {
                    lerp(self.value, *next, t)
                }
            }
            None => self.value,
        }
    }
}
