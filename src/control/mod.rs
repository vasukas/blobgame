use self::{level::LevelPlugin, menu::MenuPlugin, time::TimePlugin};
use crate::common::*;

pub mod level;
pub mod menu;
pub mod time;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TimePlugin)
            .add_plugin(LevelPlugin)
            .add_plugin(MenuPlugin);
    }
}
