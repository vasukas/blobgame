use crate::{
    common::*,
    control::{level::GameplayObject, loading::Loading},
    present::simple_sprite::{ImageVec, SimpleSprite},
};

#[derive(Component, Clone, Serialize, Deserialize)]
pub enum Spawn {}

#[derive(Component)]
pub struct SpawnActive;

//

pub struct SomePlugin;

impl Plugin for SomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .add_system_to_stage(CoreStage::First, spawn)
            .add_startup_system(load_assets);
    }
}

fn spawn(
    mut commands: Commands, tiles: Query<(&GlobalTransform, &Spawn), Added<SpawnActive>>,
    assets: Res<GameAssets>,
) {
    for (transform, spawn) in tiles.iter() {
        //
    }
}

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
