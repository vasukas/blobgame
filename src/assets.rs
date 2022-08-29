use crate::{common::*, present::simple_sprite::ImageVec};
use bevy_kira_audio::AudioSource;
use std::sync::Arc;

#[derive(Default)]
pub struct MyAssets {
    // graphics
    pub crystal: Handle<Image>,
    pub glow: Handle<Image>,
    pub player: ImageVec,

    // UI sounds (MUST NOT BE USED AS POSITIONAL)
    pub ui_menu_drone: Handle<AudioSource>,
    pub ui_pickup: Handle<AudioSource>,
    pub ui_alert: Handle<AudioSource>,
    pub beat: Handle<AudioSource>,
    //
    pub player_gun: Handle<AudioSource>,
    pub player_gun_powered: Handle<AudioSource>,

    // world sounds
    pub explosion_small: Handle<AudioSource>,
    pub wpn_smg: Handle<AudioSource>,
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

    // UI sounds
    assets.ui_menu_drone = server.load("sounds/ui/the_noise.ogg");
    assets.ui_pickup = server.load("sounds/ui/ui_pickup.ogg");
    assets.ui_alert = server.load("sounds/ui/ui_alert.ogg");
    assets.beat = server.load("sounds/ui/beat.ogg");
    //
    assets.player_gun = server.load("sounds/ui/player_gun.ogg");
    assets.player_gun_powered = server.load("sounds/ui/player_gun_powered.ogg");

    // world sounds
    assets.explosion_small = server.load("sounds/world/explosion.ogg");
    assets.wpn_smg = server.load("sounds/world/smg.ogg");
}
