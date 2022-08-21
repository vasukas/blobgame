pub use crate::{
    control::{level::GameplayObject, time::GameTime},
    present::depth::Depth,
    utils::{bevy::*, rust::*},
};
pub use bevy::{log, math::vec2, prelude::*, utils::HashMap};
pub use bevy_egui::{egui, EguiContext};
pub use bevy_rapier2d::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use std::time::Duration;

pub use bevy_prototype_lyon::prelude as bevy_lyon;

pub const END_OF_TIMES: Duration = Duration::from_secs(60 * 60 * 24 * 30); // in 30 days

//

#[derive(Clone, Copy)]
pub enum PhysicsType {
    Obstacle,
}

impl PhysicsType {
    pub fn rapier(self) -> CollisionGroups {
        let obstacles = 1;

        let (memberships, filters) = match self {
            PhysicsType::Obstacle => (obstacles, obstacles),
        };
        CollisionGroups {
            memberships,
            filters,
        }
    }
}

impl Into<InteractionGroups> for PhysicsType {
    fn into(self) -> InteractionGroups {
        self.rapier().into()
    }
}
