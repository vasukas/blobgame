use self::{editor::EditorPlugin, level::LevelPlugin, loading::LoadingPlugin, time::TimePlugin};
use crate::common::*;

pub mod editor;
pub mod level;
pub mod loading;
pub mod time;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TimePlugin)
            .add_plugin(LoadingPlugin)
            .add_plugin(EditorPlugin)
            .add_plugin(LevelPlugin);
    }
}
