use crate::common::*;

pub mod movement;
pub mod player;
pub mod spawn;

pub struct ObjectsPlugin;

impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(spawn::SpawnPlugin)
            .add_plugin(player::PlayerPlugin)
            .add_plugin(movement::MovementPlugin);
    }
}
