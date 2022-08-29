use crate::common::*;

pub mod camera;
pub mod depth;
pub mod effect;
pub mod hud_elements;
pub mod light;
pub mod sound;

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(camera::CameraPlugin)
            .add_plugin(light::LightPlugin)
            .add_plugin(depth::DepthPlugin)
            .add_plugin(sound::SoundPlugin)
            .add_plugin(effect::EffectPlugin)
            .add_plugin(hud_elements::HudElementsPlugin);
    }
}
