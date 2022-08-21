use crate::common::*;

/// Created directly by the level editor
#[derive(Component)]
pub struct LevelObject;

/// Gets respawned
#[derive(Component)]
pub struct GameplayObject;

//

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        //
    }
}
