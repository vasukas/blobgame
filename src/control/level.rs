use crate::{common::*, objects::terrain::TerrainPoint};

// TODO: this is all unneccassry convoluted

#[derive(Component)]
pub struct GameplayObject;

#[derive(Default, Serialize, Deserialize)]
pub struct Level {
    pub points: Vec<(LevelPos, LevelObject)>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct LevelPos {
    pub pos: Vec2,
    pub angle: f32,
}

#[derive(Component, Serialize, Deserialize)]
pub enum LevelObject {
    Terrain(TerrainPoint),
}

/// Event
pub enum LevelCommand {
    Respawn,
    Load { level: Option<String> },
}

//

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LevelCommand>()
            .add_system(respawn.exclusive_system());
    }
}

fn respawn(
    mut commands: Commands, mut cmds: EventReader<LevelCommand>,
    gameplays: Query<Entity, With<GameplayObject>>, level: Query<Entity, With<LevelObject>>,
) {
}
