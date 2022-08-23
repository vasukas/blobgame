use crate::{
    common::*,
    control::{
        level::{LevelCommand, LevelEvent},
        menu::UiMenuSystem,
    },
    mechanics::physics::CollectContacts,
    present::camera::WindowInfo,
};
use bevy::utils::HashSet;

#[derive(Component)]
pub struct CheckpointArea(pub u64);

#[derive(Component)]
pub struct CheckpointSpawn(pub u64);

#[derive(Component)]
pub struct ExitArea(pub String);

//

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerState>()
            .init_resource::<TextEvents>()
            .add_system(spawn_player.exclusive_system())
            .add_system(movement)
            .add_system(actions)
            .add_system(
                respawn_and_level_info
                    .before(UiMenuSystem)
                    .after(text_event),
            )
            .add_system(text_event);
    }
}

#[derive(Default)]
struct PlayerState {
    level: Option<LevelData>,
}

#[derive(Default)]
struct LevelData {
    visited_checkpoints: HashSet<Entity>,
    last_checkpoint: Vec2,

    dead: Option<Duration>,
    next: Option<(String, Duration)>,
}

#[derive(Component, Default)]
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
    keys: Res<Input<KeyCode>>, time: Res<GameTime>, player: Query<Entity, With<Player>>,
    window: Res<WindowInfo>, checkpoints: Query<(Entity, &CheckpointArea, &CollectContacts)>,
    spawns: Query<(&GlobalTransform, &CheckpointSpawn)>,
    exit: Query<(&ExitArea, &CollectContacts)>, mut texts: ResMut<TextEvents>,
    server: Res<AssetServer>,
) {
    let fade_duration = Duration::from_secs(3);

    for event in events.iter() {
        match event {
            LevelEvent::Loaded { title } => {
                pstate.level = Some(default());
                texts
                    .evs
                    .push((time.now(), title.clone(), Duration::from_secs(4)));
            }
            LevelEvent::Unloaded => pstate.level = None,
            LevelEvent::Reloaded => {
                todo!() // TODO: SPAWN PLAYER!!!
            }
        }
    }

    if let Some(level) = pstate.level.as_mut() {
        // you are dead
        if player.is_empty() {
            let start = level.dead.get_or_insert(time.now());
            let t = time.t_passed(*start, fade_duration).min(1.);
            ctx.fill_screen(
                "player.respawn.bg",
                egui::Color32::from_black_alpha((255. * t).clamp(0., 255.) as u8),
                window.size,
            );
            ctx.popup("player.respawn", vec2(0., 0.), false, |ui| {
                ui.heading("YOU DIED");
                ui.label("Press [R] to restart");
                if keys.just_pressed(KeyCode::R) {
                    level_cmd.send(LevelCommand::Reload)
                }
            });
        } else {
            level.dead = None
        }

        // first checkpoint ever
        if level.visited_checkpoints.is_empty() {
            if let Some(point) = spawns.iter().find(|p| p.1 .0 == 0) {
                level.last_checkpoint = point.0.pos_2d();
            } else {
                log::error!("THERE IS NO FIRST SPAWN POINT");
            }
        }

        if let Ok(player) = player.get_single() {
            // checkpoints
            for (entity, checkpoint, contacts) in checkpoints.iter() {
                if contacts.current.contains(&player) {
                    if level.visited_checkpoints.insert(entity) {
                        texts.evs.push((
                            time.now(),
                            "Checkpoint!".to_string(),
                            Duration::from_secs(4),
                        ));
                        if let Some(point) = spawns.iter().find(|p| p.1 .0 == checkpoint.0) {
                            level.last_checkpoint = point.0.pos_2d();
                        } else {
                            log::error!("NO SPAWN POINT FOR AREA {}", checkpoint.0);
                        }
                    }
                }
            }

            // level exit
            for (exit, contacts) in exit.iter() {
                if contacts.current.contains(&player) {
                    if level.next.is_none() {
                        level.next = Some((exit.0.clone(), time.now()))
                    }
                }
            }
        }

        // loading next level
        if let Some((name, start)) = level.next.as_ref() {
            let t = time.t_passed(*start, fade_duration).min(1.);
            ctx.fill_screen(
                "player.respawn.next",
                egui::Color32::from_white_alpha((255. * t).clamp(0., 255.) as u8),
                window.size,
            );
            if t >= 1. {
                if name == "END" {
                    todo!() // TODO: end game!!!
                } else {
                    level_cmd.send(LevelCommand::Load(
                        server.load(&format!("levels/{}.svg", name)),
                    ))
                }
            }
        }
    }
}

#[derive(Default)]
struct TextEvents {
    evs: Vec<(Duration, String, Duration)>,
}

fn text_event(mut ctx: ResMut<EguiContext>, mut texts: ResMut<TextEvents>, time: Res<GameTime>) {
    ctx.popup("player.texts", vec2(0., 0.), false, |ui| {
        texts.evs.retain_mut(|(since, text, duration)| {
            let t = time.t_passed(*since, *duration).min(1.);
            let alpha = (255. * (1. - t)).clamp(0., 255.);
            ui.colored_label(
                egui::Color32::from_rgba_unmultiplied(255, 128, 255, alpha as u8),
                text,
            );
            t < 1.
        });
    });
}
