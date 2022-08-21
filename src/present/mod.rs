use self::{camera::CameraPlugin, simple_sprite::SimpleSpritePlugin};
use crate::common::*;

pub mod camera;
pub mod depth;
pub mod simple_sprite;

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CameraPlugin).add_plugin(SimpleSpritePlugin);
    }
}
