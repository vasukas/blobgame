use crate::{common::*, present::simple_sprite::ImageVec};
use std::sync::Arc;

#[derive(Default)]
pub struct MyAssets {
    pub crystal: Handle<Image>,
    pub glow: Handle<Image>,
    pub player: ImageVec,
}

//

pub struct MyAssetsPlugin;

impl Plugin for MyAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyAssets>()
            .add_startup_system(load_assets);
    }
}

fn load_assets(mut assets: ResMut<MyAssets>, server: Res<AssetServer>) {
    assets.crystal = server.load("sprites/crystal.png");
    assets.glow = server.load("sprites/glow.png");

    for (data, prefix, count) in [(&mut assets.player, "sprites/player/circ", 3)] {
        *data = Arc::new(
            (0..count)
                .into_iter()
                .map(|index| server.load(&format!("{}{}.png", prefix, index)))
                .collect(),
        )
    }
}
