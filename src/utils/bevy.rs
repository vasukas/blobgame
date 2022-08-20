use bevy::{log, prelude::*};

// 2D transform

pub trait BevyTransform2d {
    fn new_2d(pos: Vec2, z_level: f32) -> Self;
    fn pos_2d(&self) -> Vec2;
}

pub trait BevyTransform2dMut {
    fn add_2d(&mut self, value: Vec2);
}

impl BevyTransform2d for Transform {
    fn new_2d(pos: Vec2, z_level: f32) -> Self {
        Self::from_xyz(pos.x, pos.y, z_level)
    }
    fn pos_2d(&self) -> Vec2 {
        self.translation.truncate()
    }
}

impl BevyTransform2dMut for Transform {
    fn add_2d(&mut self, value: Vec2) {
        self.translation.x += value.x;
        self.translation.y += value.y;
    }
}

impl BevyTransform2d for GlobalTransform {
    fn new_2d(pos: Vec2, z_level: f32) -> Self {
        Transform::new_2d(pos, z_level).into()
    }
    fn pos_2d(&self) -> Vec2 {
        self.translation().truncate()
    }
}

// Glam

pub trait GlamVec2 {
    fn area(&self) -> f32;
    fn in_bounds(&self, min: Self, max: Self) -> bool;
}

impl GlamVec2 for Vec2 {
    fn area(&self) -> f32 {
        self.x * self.y
    }
    fn in_bounds(&self, min: Self, max: Self) -> bool {
        self.cmpge(min).all() && self.cmplt(max).all()
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
