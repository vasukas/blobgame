use crate::{common::*, objects::spawn::SpawnObject};
use serde::{Deserialize, Serialize};

/// Resource
#[derive(Default)]
pub struct LevelControl {
    /// Which level is currently loaded (name)
    level_spawned: Option<String>,
}

impl LevelControl {
    pub fn is_game_running(&self) -> bool {
        self.level_spawned.is_some()
    }
}

/// Resource - currently loaded level. Changes are ignored.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LevelData {
    pub name: String,
    pub objects: Vec<(Vec2, f32, SpawnObject)>,
    pub tutorial_text: Option<String>,
}

/// Event
pub enum LevelCommand {
    /// Load level by name
    Load(String),
    Unload,
}

//

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        //
    }
}
