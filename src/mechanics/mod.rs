use crate::common::*;

pub mod ai;
pub mod damage;
pub mod health;
pub mod movement;
pub mod physics;

pub struct MechanicsPlugin;

impl Plugin for MechanicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(physics::PhysicsPlugin)
            .add_plugin(health::HealthPlugin)
            .add_plugin(damage::DamagePlugin)
            .add_plugin(movement::MovementPlugin)
            .add_plugin(ai::AiPlugin);
    }
}
