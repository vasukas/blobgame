use crate::{
    common::*,
    control::level::{LevelCommand, LevelEvent},
};
use bevy::utils::HashSet;

#[derive(Component)]
pub struct CheckpointArea(pub u64);

#[derive(Component)]
pub struct CheckpointSpawn(pub u64);

#[derive(Component)]
pub struct ExitArea;

//

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerState>()
            .add_system(spawn_player.exclusive_system())
            .add_system(movement)
            .add_system(actions)
            .add_system(respawn_and_level_info);
    }
}

#[derive(Default)]
struct PlayerState {
    show_level_title: Option<(String, Duration)>,
    level: Option<LevelData>,
}

#[derive(Default)]
struct LevelData {
    visited_checkpoints: HashSet<Entity>,
    last_checkpoint: Vec2,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands, player: Query<Entity, Added<Player>>) {
    for entity in player.iter() {
        commands.entity(entity);
        // TODO: implement
    }
}

fn movement() {
    // TODO: implement
}

fn actions() {
    // TODO: implement
}

fn respawn_and_level_info(
    mut ctx: ResMut<EguiContext>, mut pstate: ResMut<PlayerState>,
    mut events: EventReader<LevelEvent>, mut level_cmd: EventWriter<LevelCommand>,
    keys: Res<Input<KeyCode>>, time: Res<GameTime>,
) {
    // for event in events.iter() {
    //     match event {
    //         LevelEvent::Loaded { title } => {
    //             pstate.playing = true;
    //             pstate.show_level_title = Some((title.clone(), time.now()))
    //         }
    //         LevelEvent::Unloaded => pstate.playing = false,
    //     }
    // }

    // TODO: implement
}
