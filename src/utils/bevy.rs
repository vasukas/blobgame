use crate::common::lerp;
use bevy::{
    ecs::query::{QueryItem, WorldQuery},
    log,
    prelude::*,
};
use std::time::Duration;

// 2D transform

pub trait BevyTransform2d {
    /// Returns 2D translation (without Z)
    fn pos_2d(&self) -> Vec2;
    /// Returns 2D angle. Zero points to Y axis
    fn angle_2d(&self) -> f32;
}

pub trait BevyTransform2dMut {
    /// Sets 2D translation, Z is unchanged
    fn add_2d(&mut self, value: Vec2);
    /// Sets 2D translation, Z is unchanged
    fn set_2d(&mut self, value: Vec2);
    /// Sets 2D angle. Zero points to Y axis
    fn set_angle_2d(&mut self, value: f32);
    /// Sets same scale to X and Y
    fn set_scale_2d(&mut self, value: f32);
}

pub trait BevyTransform2dNew {
    /// Same as `Transform::from_translation(pos.extend(0.))`
    fn new_2d(pos: Vec2) -> Self;
    fn with_angle_2d(self, angle: f32) -> Self;
    fn bundle(self) -> SpatialBundle;
}

impl BevyTransform2d for Transform {
    fn pos_2d(&self) -> Vec2 {
        self.translation.truncate()
    }
    fn angle_2d(&self) -> f32 {
        self.rotation.to_euler(EulerRot::ZXY).0
    }
}

impl BevyTransform2dMut for Transform {
    fn add_2d(&mut self, value: Vec2) {
        self.translation.x += value.x;
        self.translation.y += value.y;
    }
    fn set_2d(&mut self, value: Vec2) {
        self.translation.x = value.x;
        self.translation.y = value.y;
    }
    fn set_angle_2d(&mut self, value: f32) {
        self.rotation = Quat::from_euler(EulerRot::ZXY, value, 0., 0.)
    }
    fn set_scale_2d(&mut self, value: f32) {
        self.scale.x = value;
        self.scale.y = value;
    }
}

impl BevyTransform2dNew for Transform {
    fn new_2d(pos: Vec2) -> Self {
        Self::from_translation(pos.extend(0.))
    }
    fn with_angle_2d(mut self, angle: f32) -> Self {
        self.set_angle_2d(angle);
        self
    }
    fn bundle(self) -> SpatialBundle {
        self.into()
    }
}

impl BevyTransform2d for GlobalTransform {
    fn pos_2d(&self) -> Vec2 {
        self.translation().truncate()
    }
    fn angle_2d(&self) -> f32 {
        // TODO: suboptimal
        Transform::from(*self).angle_2d()
    }
}

// Glam

pub trait GlamVec2 {
    /// `x * y`
    fn area(&self) -> f32;
    /// Angle relative to Y axis
    fn angle(&self) -> f32;

    /// Does this point lie inside rectangle (or on its borders) specified by min and max
    fn in_bounds(&self, min: Self, max: Self) -> bool;
    /// Returns copy of this vector rotated by specified angle (0 points to Y axis)
    fn rotated(&self, angle: f32) -> Self;
    /// Returns copy rotated by 90 degrees clockwise
    fn clockwise90(&self) -> Self;
}

impl GlamVec2 for Vec2 {
    fn area(&self) -> f32 {
        self.x * self.y
    }
    fn angle(&self) -> f32 {
        // TODO: why negative
        -self
            .try_normalize()
            .unwrap_or(Vec2::Y)
            .angle_between(Vec2::Y)
    }

    fn in_bounds(&self, min: Self, max: Self) -> bool {
        self.cmpge(min).all() && self.cmplt(max).all()
    }
    fn rotated(&self, angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
    }
    fn clockwise90(&self) -> Self {
        Self::new(self.y, -self.x)
    }
}

// Log

pub trait LogResult<T> {
    /// Logs error
    fn ok_or_log(self) -> Option<T>;
}

impl<T, E: std::fmt::Display> LogResult<T> for Result<T, E> {
    fn ok_or_log(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                log::error!("{}", e);
                None
            }
        }
    }
}

// Color

pub trait BevyColorExtended {
    fn with_a(self, alpha: f32) -> Self;
}

impl BevyColorExtended for Color {
    fn with_a(mut self, alpha: f32) -> Self {
        *self.set_a(alpha)
    }
}

/// Lerps both color components and alpha
pub fn lerp_color(v0: Color, v1: Color, t: f32) -> Color {
    lerp(v0, v1, t).with_a(lerp(v0.a(), v1.a(), t))
}

// Command events

pub type CmdReader<'w, 's, Event> = EventReader<'w, 's, (Entity, Event)>;
pub type CmdWriter<'w, 's, Event> = EventWriter<'w, 's, (Entity, Event)>;

pub trait CmdReaderExtended<Event> {
    /// Iterate over all entities which received an event
    fn iter_entities<'w2, 's2, Q: WorldQuery, F: WorldQuery>(
        &mut self, query: &mut Query<'w2, 's2, Q, F>, apply: impl FnMut(&Event, QueryItem<Q>),
    );
}

impl<'w, 's, Event: 'static + Send + Sync> CmdReaderExtended<Event> for CmdReader<'w, 's, Event> {
    fn iter_entities<'w2, 's2, Q: WorldQuery, F: WorldQuery>(
        &mut self, query: &mut Query<'w2, 's2, Q, F>, mut apply: impl FnMut(&Event, QueryItem<Q>),
    ) {
        for (entity, cmd) in self.iter() {
            if let Ok(data) = query.get_mut(*entity) {
                apply(cmd, data)
            }
        }
    }
}

// Time

pub trait BevyTimeExtended {
    /// Current time. Monotonically advances
    fn now(&self) -> Duration;

    /// How much time advanced last frame
    fn delta(&self) -> Duration;

    /// Note that this might be zero!
    fn delta_seconds(&self) -> f32 {
        self.delta().as_secs_f32()
    }

    /// Have reached or passed that time
    fn reached(&self, time: Duration) -> bool {
        self.now() >= time
    }

    /// How much passed since that time
    fn passed(&self, since: Duration) -> Duration {
        self.now().checked_sub(since).unwrap_or_default()
    }

    /// Returns `passed(since) / period`
    fn t_passed(&self, since: Duration, period: Duration) -> f32 {
        self.passed(since).as_secs_f32() / period.as_secs_f32()
    }

    /// Returns Some(count) if new period started this frame
    fn tick_count(&self, start: Duration, period: Duration) -> Option<u32> {
        let tick_count = |time| {
            self.now()
                .checked_sub(time)
                .map(|passed| passed.as_micros() / period.as_micros())
        };
        // get current tick count and tick count for next frame
        let current = tick_count(start);
        //let previous = tick_count(start.checked_sub(self.delta()).unwrap_or_default());
        let previous = tick_count(start + self.delta());
        // then return current tick count if counts are different
        (current != previous).then_some(current.unwrap_or(0) as u32)
    }

    /// Returns true if new period started on this frame
    fn is_tick(&self, start: Duration, period: Duration) -> bool {
        self.tick_count(start, period).is_some()
    }
}

impl BevyTimeExtended for Time {
    fn now(&self) -> Duration {
        self.time_since_startup()
    }
    fn delta(&self) -> Duration {
        self.delta()
    }
}
