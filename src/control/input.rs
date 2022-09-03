use crate::common::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

pub use leafwing_input_manager::prelude::ActionState;

/// Read with ActionState component
#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,

    Fire,
    FireMega,
    ChangeWeapon,

    Dash,
    Focus,
}

/// Read with ActionState resource
#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlAction {
    Restart,
    ExitMenu,
}

/// Read with ActionState component
#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CraftAction {
    CraftSelect1,
    CraftSelect2,
    CraftSelect3,
    CraftSelect4,
    Craft,
}

#[derive(Serialize, Deserialize)]
pub struct InputSettings {
    // TODO: implement
}

impl InputSettings {
    /// Returns true if changed
    pub fn menu(&mut self, ui: &mut egui::Ui) -> bool {
        ui.label("NOT IMPLEMENTED"); // TODO: implement
        false
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        // TODO: implement
        Self {}
    }
}

//

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default())
            .add_plugin(InputManagerPlugin::<CraftAction>::default())
            .add_plugin(InputManagerPlugin::<ControlAction>::default())
            .insert_resource(ActionState::<ControlAction>::default())
            .insert_resource(InputMap::<ControlAction>::new([
                (KeyCode::R, ControlAction::Restart),
                (KeyCode::Escape, ControlAction::ExitMenu),
                (KeyCode::M, ControlAction::ExitMenu),
            ]));
    }
}
