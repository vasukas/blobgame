use crate::{common::*, present::simple_sprite::ImageVec};
use bevy_kira_audio::AudioSource;
use std::sync::Arc;

#[derive(Default)]
pub struct MyAssets {
    // graphics
    pub glow: Handle<Image>,
    pub blob: ImageVec,

    // UI sounds (MUST NOT BE USED AS POSITIONAL)
    pub ui_menu_drone: Handle<AudioSource>,
    pub ui_pickup: Handle<AudioSource>,
    pub ui_alert: Handle<AudioSource>,
    pub beat: Handle<AudioSource>,
    pub ui_weapon_broken: Handle<AudioSource>,
    //
    pub player_gun: Handle<AudioSource>,
    pub player_gun_powered: Handle<AudioSource>,
    pub player_railgun: Handle<AudioSource>,
    pub player_plasma: Handle<AudioSource>,
    pub player_shield: Handle<AudioSource>,

    // world sounds
    pub explosion_small: Handle<AudioSource>,
    pub explosion_big: Handle<AudioSource>,
    pub wpn_smg: Handle<AudioSource>,
    pub wpn_plasma: Handle<AudioSource>,
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
    assets.glow = server.load("sprites/glow.png");

    for (data, prefix, count) in [(&mut assets.blob, "sprites/player/circ", 3)] {
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
    assets.ui_weapon_broken = server.load("sounds/ui/ui_weapon_broken.ogg");
    //
    assets.player_gun = server.load("sounds/ui/player_gun.ogg");
    assets.player_gun_powered = server.load("sounds/ui/player_gun_powered.ogg");
    assets.player_railgun = server.load("sounds/ui/player_railgun.ogg");
    assets.player_plasma = server.load("sounds/ui/plasma.ogg");
    assets.player_shield = server.load("sounds/ui/env_shield_hit.ogg");

    // world sounds
    assets.explosion_small = server.load("sounds/world/explosion.ogg");
    assets.explosion_big = server.load("sounds/world/explosion_large.ogg");
    assets.wpn_smg = server.load("sounds/world/smg.ogg");
    assets.wpn_plasma = server.load("sounds/world/plasma.ogg");
}
