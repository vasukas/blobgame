use crate::{
    common::*,
    control::{level::LevelCommand, loading::Loading},
    present::simple_sprite::{ImageVec, SimpleSprite},
};

//

#[derive(Component, Clone)]
pub enum LevelArea {
    Terrain,
    Checkpoint,
    LevelExit,
}

impl LevelArea {
    pub fn from_id(id: &str) -> Self {
        match id {
            "check" => Self::Checkpoint,
            "exit" => Self::LevelExit,
            _ => Self::Terrain,
        }
    }
}

/// If it's closed, last point will be equal to first
#[derive(Component)]
pub struct LevelAreaPolygon(pub Vec<Vec2>);

#[derive(Component, Clone)]
pub enum LevelObject {
    PlayerSpawn,
}

impl LevelObject {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "player" => Some(LevelObject::PlayerSpawn),
            _ => None,
        }
    }
}

//

pub struct SomePlugin;

impl Plugin for SomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            //.add_system_to_stage(CoreStage::First, spawn)
            .add_startup_system(load_assets);
    }
}

/* fn spawn(
    mut commands: Commands, tiles: Query<(&GlobalTransform, &Spawn), Added<SpawnActive>>,
    assets: Res<GameAssets>,
) {
    for (transform, spawn) in tiles.iter() {
        //
    }
} */

#[derive(Default)]
struct GameAssets {
    player_sprite: ImageVec,
    wall_sprite: ImageVec,
}

fn load_assets(
    mut loading: ResMut<Loading>, server: Res<AssetServer>, mut assets: ResMut<GameAssets>,
) {
    assets.player_sprite = ImageVec::new(loading.add_n(&server, "sprites/player/circ?.png", 3));
    assets.wall_sprite = ImageVec::new(loading.add_n(&server, "sprites/wall/wall?.png", 3));
}
