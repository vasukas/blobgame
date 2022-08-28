use crate::{common::*, present::simple_sprite::ImageVec};
use bevy_kira_audio::AudioSource;
use std::sync::Arc;

#[derive(Default)]
pub struct MyAssets {
    // graphics
    pub crystal: Handle<Image>,
    pub glow: Handle<Image>,
    pub player: ImageVec,

    // sounds
    pub explosion_small: Handle<AudioSource>,
    pub menu_drone: Handle<AudioSource>,
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

    assets.explosion_small = server.load("sounds/explosion_bot_1.ogg");
    assets.menu_drone = server.load("sounds/the_noise.ogg");
}
