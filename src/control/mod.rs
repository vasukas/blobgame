use self::{loading::LoadingPlugin, time::TimePlugin, editor::EditorPlugin};
use crate::common::*;

pub mod editor;
pub mod loading;
pub mod time;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TimePlugin).add_plugin(LoadingPlugin).add_plugin(EditorPlugin);
    }
}
