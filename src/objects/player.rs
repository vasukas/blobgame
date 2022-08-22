use crate::{
    common::*,
    control::level::{LevelCommand, LevelEvent},
};

#[derive(Component, Default)]
pub struct Player;

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
    playing: bool,
    show_level_title: Option<(String, Duration)>,
}

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
    for event in events.iter() {
        match event {
            LevelEvent::Loaded { title } => {
                pstate.playing = true;
                pstate.show_level_title = Some((title.clone(), time.now()))
            }
            LevelEvent::Unloaded => pstate.playing = false,
        }
    }

    // TODO: implement
}
