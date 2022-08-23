use crate::{
    common::*,
    control::{
        level::{LevelCommand, LevelEvent},
        menu::UiMenuSystem,
    },
    mechanics::physics::CollectContacts,
    present::{
        camera::{WindowInfo, WorldCameraTarget},
        simple_sprite::SimpleSprite,
    },
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
            .add_system(spawn_player)
            .add_system(movement)
            .add_system(actions)
            .add_system(respawn.before(UiMenuSystem).after(text_event))
            .add_system(progress.before(UiMenuSystem).after(text_event))
            .add_system_to_stage(CoreStage::Last, track_checkpoints)
            .add_system(track_level)
            .add_system(text_event);
    }
}

#[derive(Default)]
struct PlayerState {
    level: Option<LevelData>,
    pending_spawn: bool,

    // statistics
    deaths: usize,
}

#[derive(Default)]
struct LevelData {
    visited_checkpoints: HashSet<Entity>,
    last_checkpoint: Option<Vec2>,

    next: Option<(String, Duration)>,
    dead: Option<Duration>,
}

#[derive(Component, Default)]
struct Player {
    radius: f32,

    last_move_dir: Vec2,

    velocity: Vec2,
    on_ground: bool,
}

//

fn spawn_player(
    mut commands: Commands, mut events: EventReader<LevelEvent>, mut pstate: ResMut<PlayerState>,
    assets: Res<MyAssets>,
) {
    for event in events.iter() {
        match event {
            LevelEvent::Loaded { .. } | LevelEvent::Reloaded => pstate.pending_spawn = true,
            LevelEvent::Unloaded => (),
        }
    }
    if let Some(checkpoint) = pstate
        .pending_spawn
        .then_some(())
        .and_then(|_| pstate.level.as_ref())
        .and_then(|level| level.last_checkpoint)
    {
        let radius = 0.5;

        use bevy_lyon::*;
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shapes::Circle {
                    radius,
                    ..default()
                },
                DrawMode::Fill(FillMode::color(Color::rgb(0.4, 0.9, 1.).with_a(0.1))),
                Transform::from_translation(checkpoint.extend(0.)),
            ))
            .insert(GameplayObject)
            .insert(Player {
                radius,
                last_move_dir: vec2(1., 0.),
                ..default()
            })
            //
            .insert(Depth::Player)
            .insert(SimpleSprite {
                images: assets.player.clone(),
                frame_duration: Duration::from_millis(250),
                size: Vec2::splat(radius * 2.),
                ..default()
            })
            .insert(WorldCameraTarget)
            //
            .insert(RigidBody::KinematicPositionBased)
            .insert(PhysicsType::Player.rapier())
            .insert(Collider::ball(radius));

        pstate.pending_spawn = false;
        log::info!("Player spawned at {}", checkpoint);
    }
}

fn movement(
    mut player: Query<(Entity, &mut Transform, &mut Player)>, keys: Res<Input<KeyCode>>,
    window: Res<WindowInfo>, time: Res<GameTime>, phy_config: Res<RapierConfiguration>,
    phy: Res<RapierContext>,
) {
    let movement_speed = 10.;

    let (entity, mut transform, mut player) = match player.get_single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };

    let mut move_dir = Vec2::ZERO;
    if keys.pressed(KeyCode::W) {
        move_dir.y += 1.
    }
    if keys.pressed(KeyCode::S) {
        move_dir.y -= 1.
    }
    if keys.pressed(KeyCode::A) {
        move_dir.x -= 1.
    }
    if keys.pressed(KeyCode::D) {
        move_dir.x += 1.
    }
    let move_dir = move_dir.try_normalize();

    //

    let mut velocity = player.velocity;
    if let Some(move_dir) = move_dir {
        player.last_move_dir = move_dir;
        velocity += move_dir * movement_speed;
    }
    if player.on_ground {
        player.velocity = Vec2::ZERO
    } else {
        player.velocity += phy_config.gravity * time.delta_seconds();
    }

    let origin = transform.pos_2d();
    let filter = QueryFilter::new()
        .exclude_rigid_body(entity)
        .groups(PhysicsType::Player.into())
        .exclude_sensors();
    let dir = velocity.normalize_or_zero();

    if let Some((_, distance)) = phy.cast_ray(
        origin,
        dir,
        player.radius,
        false,
        filter,
    ) {
        player.on_ground = true;
        transform.add_2d(dir * (distance - player.radius));
    } else {
        transform.add_2d(velocity * time.delta_seconds());
    }
}

fn actions() {
    // TODO: implement
}

//

fn track_checkpoints(
    mut pstate: ResMut<PlayerState>,
    checkpoints: Query<(Entity, &CheckpointArea, &CollectContacts)>,
    spawns: Query<(&GlobalTransform, &CheckpointSpawn)>, player: Query<Entity, With<Player>>,
    mut texts: ResMut<TextEvents>, time: Res<GameTime>,
) {
    let pstate = &mut *pstate;
    if let Some(level) = pstate.level.as_mut() {
        // first checkpoint
        if level.last_checkpoint.is_none() {
            match spawns.iter().find(|p| p.1 .0 == 0) {
                Some(point) => level.last_checkpoint = Some(point.0.pos_2d()),
                None => panic!("THERE IS NO FIRST SPAWN POINT"),
            }
        }

        // new checkpoints
        if let Ok(player) = player.get_single() {
            for (entity, checkpoint, contacts) in checkpoints.iter() {
                if contacts.current.contains(&player) {
                    if level.visited_checkpoints.insert(entity) {
                        texts.evs.push((
                            time.now(),
                            "Checkpoint!".to_string(),
                            Duration::from_secs(4),
                        ));
                        if let Some(point) = spawns.iter().find(|p| p.1 .0 == checkpoint.0) {
                            level.last_checkpoint = Some(point.0.pos_2d());
                        } else {
                            log::error!("NO SPAWN POINT FOR AREA {}", checkpoint.0);
                        }
                    }
                }
            }
        }
    }
}

fn track_level(
    mut pstate: ResMut<PlayerState>, mut events: EventReader<LevelEvent>,
    exit: Query<(&ExitArea, &CollectContacts)>, player: Query<Entity, With<Player>>,
    mut texts: ResMut<TextEvents>, time: Res<GameTime>,
) {
    // set level state
    for event in events.iter() {
        match event {
            LevelEvent::Loaded { title } => {
                pstate.level = Some(default());
                texts
                    .evs
                    .push((time.now(), title.clone(), Duration::from_secs(4)));
            }
            LevelEvent::Unloaded => pstate.level = None,
            LevelEvent::Reloaded => (),
        }
    }

    // check if touches level exit
    let pstate = &mut *pstate;
    if let Some(level) = pstate.level.as_mut() {
        if let Ok(player) = player.get_single() {
            for (exit, contacts) in exit.iter() {
                if contacts.current.contains(&player) {
                    if level.next.is_none() {
                        level.next = Some((exit.0.clone(), time.now()))
                    }
                }
            }
        }
    }
}

fn respawn(
    mut ctx: ResMut<EguiContext>, mut pstate: ResMut<PlayerState>, time: Res<GameTime>,
    mut level_cmd: EventWriter<LevelCommand>, player: Query<(), With<Player>>,
    window: Res<WindowInfo>, keys: Res<Input<KeyCode>>,
) {
    let fade_duration = Duration::from_secs(3);

    if let Some(level) = pstate.level.as_mut() {
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
                    level_cmd.send(LevelCommand::Reload);
                    pstate.deaths += 1;
                }
            });
        } else {
            level.dead = None
        }
    }
}

fn progress(
    mut ctx: ResMut<EguiContext>, mut pstate: ResMut<PlayerState>, time: Res<GameTime>,
    window: Res<WindowInfo>, mut level_cmd: EventWriter<LevelCommand>, server: Res<AssetServer>,
) {
    let fade_duration = Duration::from_secs(3);

    if let Some(level) = pstate.level.as_mut() {
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
