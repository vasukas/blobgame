use super::{
    loot::LootPicker,
    spawn::{SpawnControl, WaveEvent},
    stats::Stats,
    weapon::{CraftedWeapon, Weapon},
};
use crate::{
    common::*,
    control::{input::*, menu::TimeMode},
    mechanics::{
        damage::{BonkToTeam, Team},
        health::{Health, ReceivedDamage},
        movement::*,
    },
    present::{
        camera::WindowInfo,
        effect::{Explosion, Flash, FlashOnDamage},
        hud_elements::WorldText,
        sound::{AudioListener, Beats, Sound},
    },
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, spawn_player.exclusive_system())
            .add_system(controls.before(MovementSystemLabel))
            .add_system(respawn)
            .add_system(update_player)
            .add_system(player_damage_reaction)
            .add_system(next_wave.exclusive_system())
            .add_system(hud_panel)
            .add_system(god_mode);
    }
}

#[derive(Component, Default)]
pub struct Player {
    exhaustion: f32,
    dash_until: Option<Duration>,
    prev_move: Vec2,
    beats_count: Option<i32>,
    fire_lock: Option<(Duration, bool)>,
}

impl Player {
    pub const RADIUS: f32 = 0.6;
    const MAX_EXHAUSTION: f32 = 3.;
    pub const DASH_DISTANCE: f32 = 5.;
    const DASH_DURATION: Duration = Duration::from_millis(250);
    const SPEED: f32 = 7.;

    fn exhaust(&mut self, value: f32) -> bool {
        if self.exhaustion + value <= Player::MAX_EXHAUSTION {
            self.exhaustion += value;
            true
        } else {
            false
        }
    }

    fn try_shoot(&mut self, time: &Time, mega: bool) -> bool {
        let duration = Duration::from_millis(200);
        let can_shoot = self
            .fire_lock
            .map(|(start, was_mega)| {
                time.time_since_startup() - start >= duration || mega != was_mega
            })
            .unwrap_or(true);
        if can_shoot {
            self.fire_lock = Some((time.time_since_startup(), mega))
        }
        can_shoot
    }

    fn add_beats(&mut self, current_level: usize) {
        let add_beats = if current_level == 0 { 4 } else { 8 };
        *self.beats_count.get_or_insert(0) += add_beats;
    }
}

fn spawn_player(mut commands: Commands, player: Query<Entity, Added<Player>>) {
    for entity in player.iter() {
        let radius = Player::RADIUS;

        commands
            .entity(entity)
            .insert(KinematicController {
                speed: Player::SPEED,
                radius,
                dash_distance: Player::DASH_DISTANCE,
                dash_duration: Player::DASH_DURATION,
                ..default()
            })
            .insert(RigidBody::KinematicPositionBased)
            .insert(Collider::ball(radius * 0.6))
            .insert(PhysicsType::Solid.rapier())
            //
            .insert(Depth::Player)
            .insert(FlashOnDamage::Radius(Player::RADIUS))
            .with_children(|parent| {
                use bevy_lyon::*;
                parent.spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Polygon {
                        points: vec![
                            vec2(-radius * 0.4, radius * 0.2),
                            vec2(0., radius),
                            vec2(radius * 0.4, radius * 0.2),
                            //
                            vec2(radius * 0.7, radius * 0.5),
                            vec2(radius, 0.),
                            vec2(radius * 0.7, -radius),
                            vec2(radius * 0.3, -radius * 0.4),
                            //
                            vec2(-radius * 0.3, -radius * 0.4),
                            vec2(-radius * 0.7, -radius),
                            vec2(-radius, 0.),
                            vec2(-radius * 0.7, radius * 0.5),
                        ],
                        closed: true,
                    },
                    DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::CYAN * 0.5),
                        outline_mode: StrokeMode::new(Color::WHITE, 0.04),
                    },
                    default(),
                ));
            })
            .insert(
                Explosion {
                    color0: Color::WHITE,
                    color1: Color::RED,
                    time: Duration::from_secs(1),
                    radius: 5.,
                    power: crate::present::effect::ExplosionPower::Big,
                    ..default()
                }
                .death(),
            )
            .insert(AudioListener)
            //
            .insert(Team::Player)
            .insert(Health::new(3.).armor())
            .insert(LootPicker {
                radius: Player::RADIUS,
                ..default()
            })
            .insert(BonkToTeam(Team::YEEEEEEE))
            //
            .insert_bundle(InputManagerBundle::<PlayerAction>::default());
    }
}

fn controls(
    mut player: Query<(
        Entity,
        &GlobalTransform,
        &mut Player,
        &mut KinematicController,
        &ActionState<PlayerAction>,
    )>,
    mut kinematic: CmdWriter<KinematicCommand>, window: Res<WindowInfo>, time: Res<GameTime>,
    mut commands: Commands, mut weapon: CmdWriter<Weapon>, mut stats: ResMut<Stats>,
    mut beats: ResMut<Beats>, mut time_mode: ResMut<TimeMode>, real_time: Res<Time>,
) {
    let (entity, pos, mut player, mut kctr, input) = match player.get_single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };
    let pos = pos.pos_2d();

    let mov = vec2(
        match (
            input.pressed(PlayerAction::MoveLeft),
            input.pressed(PlayerAction::MoveRight),
        ) {
            (true, false) => -1.,
            (false, true) => 1.,
            _ => 0.,
        },
        match (
            input.pressed(PlayerAction::MoveDown),
            input.pressed(PlayerAction::MoveUp),
        ) {
            (true, false) => -1.,
            (false, true) => 1.,
            _ => 0.,
        },
    );
    let dash = input.just_pressed(PlayerAction::Dash);

    if input.just_pressed(PlayerAction::Fire) && player.try_shoot(&real_time, false) {
        weapon.send((
            entity,
            Weapon::PlayerGun {
                dir: window.cursor - pos,
            },
        ))
    }
    if input.just_pressed(PlayerAction::FireMega) && player.try_shoot(&real_time, true) {
        weapon.send((
            entity,
            Weapon::PlayerCrafted {
                dir: window.cursor - pos,
            },
        ))
    }
    if input.just_pressed(PlayerAction::Focus) && stats.ubercharge >= 1. {
        stats.ubercharge = 0.;

        if beats.level == 0 {
            beats.level = 1;
        } else {
            beats.level = 2;
        };

        player.add_beats(beats.level);
        kctr.speed = Player::SPEED * 2.;
        time_mode.overriden = Some(0.5);
    }
    if input.just_pressed(PlayerAction::ChangeWeapon) {
        let slot = &mut stats.player_weapon_slot;
        *slot = if *slot == 1 { 0 } else { 1 }
    }

    if let Some(dir) = mov.try_normalize() {
        player.prev_move = dir;
        kinematic.send((entity, KinematicCommand::Move { dir }))
    }
    if dash && player.dash_until.is_none() && player.exhaust(1.) {
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
                dir: player.prev_move,
            },
        ));
    }
}

fn respawn(
    mut ctx: ResMut<EguiContext>, player: Query<With<Player>>, mut spawn: ResMut<SpawnControl>,
    input: Res<ActionState<ControlAction>>, input_map: ResMut<InputMap<ControlAction>>,
) {
    if !spawn.is_game_running() {
        return;
    }
    if player.is_empty() {
        ctx.popup(
            "player::respawn",
            Vec2::ZERO,
            true,
            egui::Order::Background,
            |ui| {
                ui.heading("~= YOU DIED =~");

                ui.horizontal(|ui| {
                    // TODO: where should be a better way to make multicolored text
                    ui.label("Press [");
                    ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                    ui.label(input_map.prompt(ControlAction::Restart));
                    ui.visuals_mut().override_text_color = None;
                    ui.label("] to restart level");
                });
            },
        );
        if input.just_pressed(ControlAction::Restart) {
            spawn.despawn = Some(true)
        }
    }
}

fn update_player(
    mut player: Query<(
        &mut Player,
        &mut Health,
        &mut KinematicController,
        &mut Transform,
    )>,
    time: Res<GameTime>, mut beats: ResMut<Beats>, mut time_mode: ResMut<TimeMode>,
    mut stats: ResMut<Stats>, spawn: Res<SpawnControl>, window: Res<WindowInfo>,
) {
    let exhaust_restore_speed = 1.;
    let charge_time_seconds = 6.;

    for (mut player, mut health, _, mut transform) in player.iter_mut() {
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

        // increase charge
        if (!spawn.waiting_for_next_wave || spawn.tutorial.is_some()) && beats.level == 0 {
            stats.ubercharge += time.delta_seconds() / charge_time_seconds;
        }

        // rotate
        let angle = (window.cursor - transform.pos_2d()).angle();
        transform.set_angle_2d(angle);
    }

    // beats
    if player
        .get_single()
        .ok()
        .and_then(|v| v.0.beats_count.map(|count| beats.count >= count))
        .unwrap_or(true)
    {
        if let Some((mut player, ..)) = player
            .get_single_mut()
            .ok()
            .filter(|v| stats.ubercharge >= 1. && v.0.beats_count.is_some())
        {
            stats.ubercharge = 0.;
            player.add_beats(beats.level);
        } else {
            beats.level = 0;
            time_mode.overriden = None;

            if let Ok((mut player, _, mut kctr, _)) = player.get_single_mut() {
                kctr.speed = Player::SPEED;
                player.beats_count = None
            }
        }
    }
}

fn player_damage_reaction(
    mut player: Query<(Entity, &Health, &mut Player)>, mut events: CmdReader<ReceivedDamage>,
    mut was_damaged: Local<bool>, mut sound: EventWriter<Sound>, assets: Res<MyAssets>,
    mut stats: ResMut<Stats>,
) {
    let charge_loss_on_hit = 0.1;

    events.iter_entities(&mut player, |_, (_, _, mut player)| {
        if let Some(beats) = player.beats_count.as_mut() {
            *beats -= 1
        }
        stats.ubercharge = (stats.ubercharge.min(1.) - charge_loss_on_hit).max(0.);
    });

    let damaged = player
        .get_single()
        .map(|v| v.1.value < v.1.max / 2.)
        .unwrap_or(false);
    if damaged != *was_damaged {
        *was_damaged = damaged;
        if damaged {
            sound.send(Sound {
                sound: assets.ui_alert.clone(),
                non_randomized: true,
                ..default()
            })
        }
    }
}

#[derive(Default)]
struct NextWaveMenu {
    text: Option<Entity>,
}

fn next_wave(
    mut wave: EventReader<WaveEvent>, mut stats: ResMut<Stats>, mut spawn: ResMut<SpawnControl>,
    mut commands: Commands, input: Res<ActionState<ControlAction>>, mut data: Local<NextWaveMenu>,
    input_map: Res<InputMap<ControlAction>>,
) {
    // begin user input
    if wave.iter().any(|ev| *ev == WaveEvent::Ended) {
        let text = match spawn.tutorial {
            Some(1) => vec![(
                "This is tutorial\nRead text at the bottom of screen".to_string(),
                Color::WHITE,
            )],
            // THE UGLY HACK. this is # of tutorial wave + 1
            Some(4) | Some(7) | None => vec![
                ("Press [".to_string(), Color::WHITE),
                (input_map.prompt(ControlAction::Restart), Color::RED),
                ("] to go to next level".to_string(), Color::WHITE),
            ],
            Some(_) => vec![],
        };
        if let Some(text) = data.text {
            commands.entity(text).despawn_recursive();
        }
        data.text = Some(
            commands
                .spawn_bundle(SpatialBundle::default())
                .insert(WorldText { text, size: 2. })
                .id(),
        );
    } else if let Some(text) = data.text {
        // waiting for user input
        if spawn.is_game_running() {
            if input.just_pressed(ControlAction::Restart) {
                if spawn.tutorial.is_none() {
                    stats.wave += 1;
                }
                spawn.despawn = Some(true);

                commands.entity(text).despawn_recursive();
                data.text = None;
            }
        } else {
            // not anymore
            commands.entity(text).despawn_recursive();
            data.text = None;
        }
    }
}

fn hud_panel(
    mut ctx: ResMut<EguiContext>, stats: Res<Stats>, player: Query<(&Health, &Player)>,
    beats: Res<Beats>,
) {
    ctx.popup(
        "player::hud_panel",
        vec2(-1., -1.),
        false,
        egui::Order::Background,
        |ui| {
            ui.label("LEVEL");
            ui.label(format!("{}", stats.wave + 1));
            ui.label("");

            if let Ok((health, player)) = player.get_single() {
                ui.label("HEALTH");
                let hp = (health.value / health.max * 100.).clamp(0., 100.) as u32;
                ui.visuals_mut().override_text_color = Some(if hp < 50 {
                    egui::Color32::RED
                } else if hp < 80 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::GREEN
                });
                ui.label(format!("{:3}", hp));
                ui.visuals_mut().override_text_color = None;
                ui.label("");

                ui.label("STAMINA");
                let stamina = (Player::MAX_EXHAUSTION - player.exhaustion) as u32;
                ui.visuals_mut().override_text_color = Some(match stamina {
                    0 => egui::Color32::DARK_GRAY,
                    1 => egui::Color32::YELLOW,
                    _ => egui::Color32::GREEN,
                });
                ui.label(format!("{}", stamina));
                ui.visuals_mut().override_text_color = None;
                ui.label("");

                ui.label("POINTS");
                ui.label(format!("{}", stats.player.points));
                ui.label("");

                ui.label("TIME");
                ui.label(format!(
                    "{:02}:{:02}",
                    stats.time.as_secs() / 60,
                    stats.time.as_secs() % 60
                ));
                ui.label("");

                ui.label("WEAPONS");
                for v in &stats.player.weapons {
                    if let Some((weapon, uses)) = v {
                        let (name, _, max_uses) = weapon.description();
                        ui.label(format!("{} {}%", name, (uses / max_uses * 100.) as u32));
                    } else {
                        ui.label("empty");
                    }
                }
                ui.label("");

                match player.beats_count {
                    Some(count) => {
                        let left = count - beats.count;
                        ui.label("BEATS");
                        ui.visuals_mut().override_text_color = match stats.ubercharge >= 1. {
                            true => Some(egui::Color32::WHITE),
                            false => (left <= 2).then_some(egui::Color32::RED),
                        };
                        ui.label(format!("{:2} left", left));
                    }
                    None => {
                        ui.label("CHARGE");
                        ui.visuals_mut().override_text_color = match stats.ubercharge >= 1. {
                            true => Some(egui::Color32::WHITE),
                            false => None,
                        };
                        ui.label(format!(
                            "{:3}%",
                            (stats.ubercharge * 100.).clamp(0., 100.) as u32
                        ));
                    }
                }
                ui.visuals_mut().override_text_color = None;
                ui.label("");
            } else {
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.label("DEAD");
            }
        },
    );
}

fn god_mode(
    keys: Res<Input<KeyCode>>, mut player: Query<&mut Health, With<Player>>,
    mut stats: ResMut<Stats>,
) {
    if keys.just_pressed(KeyCode::P) {
        if let Ok(mut health) = player.get_single_mut() {
            health.invincible = true
        }
        *stats.weapon_mut() = Some((CraftedWeapon::Railgun, 100000.))
    }
}
