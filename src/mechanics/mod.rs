use crate::common::*;

pub mod combat;
pub mod physics;

pub struct MechanicsPlugin;

impl Plugin for MechanicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(physics::PhysicsPlugin)
            .add_plugin(combat::CombatPlugin);
    }
}
