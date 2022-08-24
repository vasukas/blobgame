use crate::common::*;

pub mod debug;
pub mod menu;
pub mod time;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(time::TimePlugin)
            .add_plugin(menu::MenuPlugin)
            .add_plugin(debug::DebugPlugin);
    }
}
