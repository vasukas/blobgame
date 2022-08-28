use crate::common::*;
use enum_map::{enum_map, Enum, EnumMap};
use serde::{Deserialize, Serialize};

/// Event
#[derive(Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum InputAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,

    Fire,
    FireMega,
    ChangeWeapon,
    Craft,

    CraftSelect1,
    CraftSelect2,
    CraftSelect3,
    CraftSelect4,

    UberCharge,
    Dash,
    TargetDash,
    Respawn,
}

impl InputAction {
    pub fn description(&self) -> &'static str {
        match self {
            InputAction::MoveLeft => "Move left",
            InputAction::MoveRight => "Move right",
            InputAction::MoveUp => "Move up",
            InputAction::MoveDown => "Move down",

            InputAction::Fire => "Fire",
            InputAction::FireMega => "Fire Megagun",
            InputAction::ChangeWeapon => "Change weapon",
            InputAction::Craft => "Craft weapon",

            InputAction::CraftSelect1 => "Select craft part 1",
            InputAction::CraftSelect2 => "Select craft part 2",
            InputAction::CraftSelect3 => "Select craft part 3",
            InputAction::CraftSelect4 => "Select craft part 4",

            InputAction::UberCharge => "Ubercharge",
            InputAction::Dash => "Dash",
            InputAction::TargetDash => "Dash to cursor",
            InputAction::Respawn => "Retry",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum InputKey {
    Key(KeyCode),
    Button(MouseButton),
}

impl ToString for InputKey {
    fn to_string(&self) -> String {
        match self {
            InputKey::Key(key) => format!("{:?}", key),
            InputKey::Button(key) => format!("{:?}", key),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputType {
    Click,
    Hold,
}

impl InputType {
    pub fn description(&self) -> &'static str {
        match self {
            InputType::Click => "Click",
            InputType::Hold => "Hold",
        }
    }
}

/// Resource
#[derive(Clone, Serialize, Deserialize)]
pub struct InputMap {
    pub map: EnumMap<InputAction, (InputKey, InputType)>,
}

impl Default for InputMap {
    fn default() -> Self {
        use InputAction::*;
        Self {
            map: enum_map! {
                MoveLeft => (InputKey::Key(KeyCode::A), InputType::Hold),
                MoveRight => (InputKey::Key(KeyCode::D), InputType::Hold),
                MoveUp => (InputKey::Key(KeyCode::W), InputType::Hold),
                MoveDown => (InputKey::Key(KeyCode::S), InputType::Hold),

                InputAction::Fire => (InputKey::Button(MouseButton::Left), InputType::Click),
                InputAction::FireMega => (InputKey::Button(MouseButton::Right), InputType::Click),
                InputAction::ChangeWeapon => (InputKey::Key(KeyCode::F), InputType::Click),
                InputAction::Craft => (InputKey::Key(KeyCode::C), InputType::Click),

                InputAction::CraftSelect1 => (InputKey::Key(KeyCode::Key1), InputType::Click),
                InputAction::CraftSelect2 => (InputKey::Key(KeyCode::Key2), InputType::Click),
                InputAction::CraftSelect3 => (InputKey::Key(KeyCode::Key3), InputType::Click),
                InputAction::CraftSelect4 => (InputKey::Key(KeyCode::Key4), InputType::Click),

                InputAction::UberCharge => (InputKey::Key(KeyCode::H), InputType::Click),
                InputAction::Dash => (InputKey::Key(KeyCode::LShift), InputType::Click),
                InputAction::TargetDash => (InputKey::Key(KeyCode::Space), InputType::Click),
                Respawn => (InputKey::Key(KeyCode::R), InputType::Click),
            },
        }
    }
}

/// Resource - prevent action emit
#[derive(Default)]
pub struct InputLock {
    pub active: bool,
    pub allow_craft: bool, // TODO: another horrible hack
}

//

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputMap>()
            .init_resource::<InputLock>()
            .add_event::<InputAction>()
            .add_system_to_stage(CoreStage::PreUpdate, emit_action);
    }
}

fn emit_action(
    lock: Res<InputLock>, map: Res<InputMap>, mut actions: EventWriter<InputAction>,
    keys: Res<Input<KeyCode>>, buttons: Res<Input<MouseButton>>,
) {
    for (action, (key, ty)) in map.map.iter() {
        if match action {
            InputAction::Craft
            | InputAction::CraftSelect1
            | InputAction::CraftSelect2
            | InputAction::CraftSelect3
            | InputAction::CraftSelect4 => lock.active && !lock.allow_craft,
            _ => lock.active,
        } {
            continue;
        }

        let active = match key {
            InputKey::Key(key) => match ty {
                InputType::Click => keys.just_pressed(*key),
                InputType::Hold => keys.pressed(*key),
            },
            InputKey::Button(button) => match ty {
                InputType::Click => buttons.just_pressed(*button),
                InputType::Hold => buttons.pressed(*button),
            },
        };
        if active {
            actions.send(action)
        }
    }
}
