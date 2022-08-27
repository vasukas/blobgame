use super::{
    spawn::{SpawnControl, WaveEnded},
    stats::Stats,
    weapon::Weapon,
};
use crate::{
    common::*,
    control::input::{InputAction, InputMap},
    mechanics::{damage::Team, health::Health, movement::*},
    present::{
        camera::WindowInfo, effect::Flash, simple_sprite::SimpleSprite, sound::AudioListener,
    },
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, spawn_player.exclusive_system())
            .add_system(controls.before(MovementSystemLabel))
            .add_system(respawn)
            .add_system(update_player)
            .add_system(next_wave.exclusive_system());
    }
}

#[derive(Component, Default)]
pub struct Player {
    exhaustion: f32,
    dash_until: Option<Duration>,
    prev_move: Vec2,
}

impl Player {
    pub const RADIUS: f32 = 0.5;
    const MAX_EXHAUSTION: f32 = 3.;
    pub const DASH_DISTANCE: f32 = 4.5;
    const DASH_DURATION: Duration = Duration::from_millis(250);

    fn exhaust(&mut self, value: f32) -> bool {
        if self.exhaustion + value <= Player::MAX_EXHAUSTION {
            self.exhaustion += value;
            true
        } else {
            false
        }
    }
}

fn spawn_player(
    mut commands: Commands, player: Query<Entity, Added<Player>>, assets: Res<MyAssets>,
) {
    for entity in player.iter() {
        let radius = Player::RADIUS;

        commands
            .entity(entity)
            .insert(KinematicController {
                speed: 7.,
                radius,
                dash_distance: Player::DASH_DISTANCE,
                dash_duration: Player::DASH_DURATION,
                ..default()
            })
            .insert(RigidBody::KinematicPositionBased)
            .insert(Collider::ball(radius * 0.9))
            .insert(PhysicsType::Solid.rapier())
            //
            .insert(Depth::Player)
            .insert(SimpleSprite {
                images: assets.player.clone(),
                frame_duration: Duration::from_millis(250),
                size: Vec2::splat(radius * 2.),
                ..default()
            })
            .with_children(|parent| {
                use bevy_lyon::*;
                parent.spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Circle {
                        radius: radius * 0.9,
                        center: Vec2::ZERO,
                    },
                    DrawMode::Fill(FillMode::color(Color::CYAN * 0.5)),
                    default(),
                ));
            })
            .insert(AudioListener)
            //
            .insert(Team::Player)
            // TODO: increase health back
            .insert(Health::new(3.).armor());
    }
}

fn controls(
    mut player: Query<(Entity, &GlobalTransform, &mut Player)>,
    mut input: EventReader<InputAction>, mut kinematic: CmdWriter<KinematicCommand>,
    window: Res<WindowInfo>, time: Res<GameTime>, mut commands: Commands,
    mut weapon: CmdWriter<Weapon>,
) {
    let (entity, pos, mut player) = match player.get_single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };
    let pos = pos.pos_2d();

    let mut mov = Vec2::ZERO;
    for action in input.iter() {
        match action {
            InputAction::MoveLeft => mov.x -= 1.,
            InputAction::MoveRight => mov.x += 1.,
            InputAction::MoveUp => mov.y += 1.,
            InputAction::MoveDown => mov.y -= 1.,

            InputAction::Dash | InputAction::TargetDash => {
                if player.dash_until.is_none() && player.exhaust(1.) {
                    player.dash_until = Some(time.now() + Player::DASH_DURATION);
                    commands.entity(entity).insert(Flash {
                        radius: Player::RADIUS,
                        duration: Player::DASH_DURATION,
                        color0: Color::WHITE,
                        color1: Color::rgb(0.8, 1., 1.),
                    });
                    kinematic.send((
                        entity,
                        KinematicCommand::Dash {
                            dir: if *action == InputAction::Dash {
                                player.prev_move
                            } else {
                                window.cursor - pos
                            },
                            exact: *action == InputAction::TargetDash,
                        },
                    ));
                }
            }

            InputAction::Fire => weapon.send((
                entity,
                Weapon::PlayerGun {
                    dir: window.cursor - pos,
                },
            )),

            _ => (),
        }
    }
    if let Some(dir) = mov.try_normalize() {
        player.prev_move = dir;
        kinematic.send((entity, KinematicCommand::Move { dir }))
    }
}

#[derive(Default)]
struct RespawnMenu {
    death_start: Option<Duration>,
}

fn respawn(
    mut ctx: ResMut<EguiContext>, mut data: Local<RespawnMenu>, player: Query<With<Player>>,
    time: Res<Time>, mut spawn: ResMut<SpawnControl>, mut input: EventReader<InputAction>,
    input_map: Res<InputMap>, window: Res<WindowInfo>,
) {
    if !spawn.is_game_running() {
        return;
    }
    if player.is_empty() {
        let passed =
            time.time_since_startup() - *data.death_start.get_or_insert(time.time_since_startup());
        let t = passed.as_secs_f32() / Duration::from_secs(2).as_secs_f32();
        ctx.fill_screen(
            "player::respawn.bg",
            egui::Color32::from_black_alpha((t * 255.).min(255.) as u8),
            egui::Order::Background,
            window.size,
        );
        ctx.popup(
            "player::respawn",
            Vec2::ZERO,
            false,
            egui::Order::Middle,
            |ui| {
                ui.heading("~= YOU DIED =~");

                ui.horizontal(|ui| {
                    // TODO: where should be a better way to make multicolored text
                    ui.label("Press [");
                    ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                    ui.label(input_map.map[InputAction::Respawn].0.to_string());
                    ui.visuals_mut().override_text_color = None;
                    ui.label("] to restart from checkpoint");
                });
            },
        );
        for action in input.iter() {
            match action {
                InputAction::Respawn => spawn.despawn = Some(true),
                _ => (),
            }
        }
    } else {
        data.death_start = None;
    }
}

fn update_player(mut player: Query<(&mut Player, &mut Health)>, time: Res<GameTime>) {
    let exhaust_restore_speed = 1.;

    for (mut player, mut health) in player.iter_mut() {
        // dash
        if let Some(until) = player.dash_until {
            let still_dashing = !time.reached(until);
            health.invincible = still_dashing;
            if !still_dashing {
                player.dash_until = None;
            }
        }

        // reduce exhaustion
        player.exhaustion =
            (player.exhaustion - time.delta_seconds() * exhaust_restore_speed).max(0.);
    }
}

fn next_wave(
    mut wave: EventReader<WaveEnded>, mut stats: ResMut<Stats>, mut spawn: ResMut<SpawnControl>,
    mut commands: Commands, mut input: EventReader<InputAction>, input_map: Res<InputMap>,
) {
}
