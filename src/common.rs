pub use crate::{
    assets::MyAssets,
    control::time::GameTime,
    mechanics::physics::PhysicsType,
    objects::spawn::GameplayObject,
    present::depth::Depth,
    settings::{Difficulty, Settings},
    utils::{bevy::*, bevy_egui::*, math::*, rust::*},
};
pub use bevy::{log, math::vec2, prelude::*, utils::HashMap};
pub use bevy_egui::{egui, EguiContext};
pub use bevy_prototype_lyon::prelude as bevy_lyon;
pub use bevy_rapier2d::prelude::*;
pub use std::{f32::consts::TAU, time::Duration};

//

/// Really ugly pattern I should remove.
/// Someday.
/// Maybe.
#[derive(Default)]
pub struct BadEntityHack(Option<Entity>);

impl BadEntityHack {
    pub fn set(&mut self, entity: Entity) {
        self.0 = Some(entity)
    }
    pub fn get(self) -> Entity {
        self.0.unwrap()
    }
}
