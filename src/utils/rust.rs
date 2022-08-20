use std::time::Duration;

pub trait DurationExtended {
    fn div_duration_f32(&self, rhs: Duration) -> f32;
}

impl DurationExtended for Duration {
    fn div_duration_f32(&self, rhs: Duration) -> f32 {
        self.as_secs_f32() / rhs.as_secs_f32()
    }
}
