use self::{player::PlayerPlugin, spawn::SpawnPlugin};
use crate::common::*;

pub mod player;
pub mod spawn;
pub mod terrain;

pub struct ObjectsPlugin;

impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SpawnPlugin).add_plugin(PlayerPlugin);
    }
}
