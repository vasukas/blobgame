use crate::common::*;

pub mod camera;
pub mod depth;
pub mod effect;
pub mod light;
pub mod simple_sprite;
pub mod sound;

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(camera::CameraPlugin)
            .add_plugin(simple_sprite::SimpleSpritePlugin)
            .add_plugin(light::LightPlugin)
            .add_plugin(depth::DepthPlugin)
            .add_plugin(sound::SoundPlugin)
            .add_plugin(effect::EffectPlugin);
    }
}
