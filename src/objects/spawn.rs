use crate::common::*;

/// Resource
#[derive(Default)]
pub struct SpawnControl {
    /// Current state
    pub spawned: bool,

    /// Set this to Some(true) to respawn, to Some(false) to despawn
    pub despawn: Option<bool>,
}

//

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnControl>()
            .add_system_to_stage(CoreStage::First, spawn.exclusive_system());
    }
}

fn spawn(mut commands: Commands, mut control: ResMut<SpawnControl>, entities: Query<Entity>) {
    if let Some(respawn) = control.despawn.take() {
        for entity in entities.iter() {
            commands.entity(entity).despawn_recursive()
        }
        control.spawned = respawn;
        if respawn {
            // TODO: spawn stuff
        }
    }
}
