use crate::common::*;
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::*, user_input::InputKind};
use serde::{Deserialize, Serialize};

pub use leafwing_input_manager::prelude::{ActionState, InputManagerBundle, InputMap};

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

impl PlayerAction {
    pub fn description(self) -> &'static str {
        match self {
            PlayerAction::MoveLeft => "Move left",
            PlayerAction::MoveRight => "Move right",
            PlayerAction::MoveUp => "Move up",
            PlayerAction::MoveDown => "Move down",
            PlayerAction::Fire => "Fire",
            PlayerAction::FireMega => "Fire alt",
            PlayerAction::ChangeWeapon => "Change weapon",
            PlayerAction::Dash => "Dash",
            PlayerAction::Focus => "Focus mode",
        }
    }
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

impl CraftAction {
    pub fn description(self) -> &'static str {
        match self {
            CraftAction::CraftSelect1 => "Select part 1",
            CraftAction::CraftSelect2 => "Select part 2",
            CraftAction::CraftSelect3 => "Select part 3",
            CraftAction::CraftSelect4 => "Select part 4",
            CraftAction::Craft => "Craft",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InputSettings {
    pub player: InputMap<PlayerAction>,
    pub craft: InputMap<CraftAction>,
    pub control: InputMap<ControlAction>,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            player: InputMap::new([
                (KeyCode::A, PlayerAction::MoveLeft),
                (KeyCode::D, PlayerAction::MoveRight),
                (KeyCode::W, PlayerAction::MoveUp),
                (KeyCode::S, PlayerAction::MoveDown),
                (KeyCode::F, PlayerAction::ChangeWeapon),
                (KeyCode::Space, PlayerAction::Dash),
                (KeyCode::LShift, PlayerAction::Focus),
            ])
            .insert(MouseButton::Left, PlayerAction::Fire)
            .insert(MouseButton::Right, PlayerAction::FireMega)
            .insert(MouseWheelDirection::Up, PlayerAction::ChangeWeapon)
            .insert(MouseWheelDirection::Down, PlayerAction::ChangeWeapon)
            .build(),

            craft: InputMap::new([
                (KeyCode::Key1, CraftAction::CraftSelect1),
                (KeyCode::Key2, CraftAction::CraftSelect2),
                (KeyCode::Key3, CraftAction::CraftSelect3),
                (KeyCode::Key4, CraftAction::CraftSelect4),
                (KeyCode::C, CraftAction::Craft),
            ]),

            control: InputMap::new([
                (KeyCode::R, ControlAction::Restart),
                (KeyCode::Escape, ControlAction::ExitMenu),
                (KeyCode::M, ControlAction::ExitMenu),
            ]),
        }
    }
}

pub trait InputMapExtended<A> {
    fn prompt(&self, action: A) -> String;
}

impl<A: Actionlike> InputMapExtended<A> for InputMap<A> {
    fn prompt(&self, action: A) -> String {
        self.get(action)
            .get_at(0)
            .map(UserInputExtended::description)
            .unwrap_or_else(|| "<NONE>".to_string())
    }
}

pub trait UserInputExtended {
    fn description(&self) -> String;
}

impl UserInputExtended for UserInput {
    fn description(&self) -> String {
        match self {
            UserInput::Single(InputKind::Keyboard(v)) => {
                format!("{:?}", v)
            }
            UserInput::Single(InputKind::Mouse(v)) => {
                format!("{:?} button", v)
            }
            UserInput::Single(InputKind::MouseWheel(v)) => {
                format!("Wheel {:?}", v)
            }
            UserInput::Single(InputKind::GamepadButton(v)) => {
                format!("{:?}", v)
            }
            UserInput::Single(_) => "<SINGLE>".to_string(),
            UserInput::Chord(_) => "<CHORD>".to_string(),
            UserInput::VirtualDPad(_) => "<VDPAD>".to_string(),
        }
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
            .insert_resource(InputMap::<ControlAction>::default())
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_settings.before(InputManagerSystem::Update),
            );
    }
}

fn update_settings(
    mut player: Query<&mut InputMap<PlayerAction>>, mut craft: Query<&mut InputMap<CraftAction>>,
    mut control: ResMut<InputMap<ControlAction>>, settings: Res<Settings>,
) {
    if settings.is_changed() || settings.is_added() {
        if let Ok(mut map) = player.get_single_mut() {
            *map = settings.input.player.clone()
        }
        if let Ok(mut map) = craft.get_single_mut() {
            *map = settings.input.craft.clone()
        }
        *control = settings.input.control.clone()
    }

    for mut map in player.iter_mut() {
        if map.is_added() {
            *map = settings.input.player.clone()
        }
    }
    for mut map in craft.iter_mut() {
        if map.is_added() {
            *map = settings.input.craft.clone()
        }
    }
}
