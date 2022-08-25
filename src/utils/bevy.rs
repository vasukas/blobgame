use bevy::{
    ecs::query::{QueryItem, WorldQuery},
    log,
    prelude::*,
};

// 2D transform

pub trait BevyTransform2d {
    fn new_2d(pos: Vec2) -> Self;
    fn pos_2d(&self) -> Vec2;
    fn angle_2d(&self) -> f32;
}

pub trait BevyTransform2dMut {
    fn add_2d(&mut self, value: Vec2);
    fn set_2d(&mut self, value: Vec2);
    fn set_angle_2d(&mut self, value: f32);
}

impl BevyTransform2d for Transform {
    fn new_2d(pos: Vec2) -> Self {
        Self::from_translation(pos.extend(0.))
    }
    fn pos_2d(&self) -> Vec2 {
        self.translation.truncate()
    }
    fn angle_2d(&self) -> f32 {
        self.rotation.angle_between(default())
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
        self.rotation = Quat::from_axis_angle(Vec3::Z, value);
    }
}

impl BevyTransform2d for GlobalTransform {
    fn new_2d(pos: Vec2) -> Self {
        Transform::new_2d(pos).into()
    }
    fn pos_2d(&self) -> Vec2 {
        self.translation().truncate()
    }
    fn angle_2d(&self) -> f32 {
        self.to_scale_rotation_translation()
            .1
            .angle_between(default())
    }
}

// Glam

pub trait GlamVec2 {
    fn area(&self) -> f32;
    fn angle(&self) -> f32;

    fn in_bounds(&self, min: Self, max: Self) -> bool;
    fn rotated(&self, angle: f32) -> Self;
}

impl GlamVec2 for Vec2 {
    fn area(&self) -> f32 {
        self.x * self.y
    }
    fn angle(&self) -> f32 {
        // TODO: why negative
        -self
            .try_normalize()
            .unwrap_or(Vec2::X)
            .angle_between(Vec2::X)
    }

    fn in_bounds(&self, min: Self, max: Self) -> bool {
        self.cmpge(min).all() && self.cmplt(max).all()
    }
    fn rotated(&self, angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
    }
}

// Log

pub trait LogResult<T> {
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

// Command events

pub type CmdReader<'w, 's, Event> = EventReader<'w, 's, (Entity, Event)>;
pub type CmdWriter<'w, 's, Event> = EventWriter<'w, 's, (Entity, Event)>;

pub trait CmdReaderExtended<Event> {
    /// Iterate over all existing entities which received an event
    fn iter_cmd_mut<'w2, 's2, Q: WorldQuery, F: WorldQuery>(
        &mut self, query: &mut Query<'w2, 's2, Q, F>, apply: impl FnMut(&Event, QueryItem<Q>),
    );
}

impl<'w, 's, Event: 'static + Send + Sync> CmdReaderExtended<Event> for CmdReader<'w, 's, Event> {
    fn iter_cmd_mut<'w2, 's2, Q: WorldQuery, F: WorldQuery>(
        &mut self, query: &mut Query<'w2, 's2, Q, F>, mut apply: impl FnMut(&Event, QueryItem<Q>),
    ) {
        for (entity, cmd) in self.iter() {
            if let Ok(data) = query.get_mut(*entity) {
                apply(cmd, data)
            }
        }
    }
}
