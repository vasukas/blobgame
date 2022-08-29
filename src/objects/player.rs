use super::{
    loot::{CraftPart, LootPicker},
    spawn::{SpawnControl, WaveEvent},
    stats::Stats,
    weapon::{CraftedWeapon, Weapon},
};
use crate::{
    common::*,
    control::{
        input::{InputAction, InputMap},
        menu::is_exit_menu,
        time::TimeMode,
    },
    mechanics::{
        damage::{BonkToTeam, Team},
        health::{Health, ReceivedDamage},
        movement::*,
    },
    present::{
        camera::WindowInfo,
        effect::{Flash, FlashOnDamage},
        hud_elements::WorldText,
        sound::{AudioListener, Beats, Sound},
    },
    settings::Difficulty,
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
            .add_system(craft_menu);
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

fn spawn_player(
    mut commands: Commands, player: Query<Entity, Added<Player>>, settings: Res<Settings>,
) {
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
            .insert(AudioListener)
            //
            .insert(Team::Player)
            .insert(
                Health::new(match settings.difficulty {
                    Difficulty::Easy => 8.,
                    Difficulty::Hard => 3.,
                })
                .armor(),
            )
            .insert(LootPicker {
                radius: Player::RADIUS,
                ..default()
            })
            .insert(BonkToTeam(Team::YEEEEEEE));
    }
}

fn controls(
    mut player: Query<(
        Entity,
        &GlobalTransform,
        &mut Player,
        &mut KinematicController,
    )>,
    mut input: EventReader<InputAction>, mut kinematic: CmdWriter<KinematicCommand>,
    window: Res<WindowInfo>, time: Res<GameTime>, mut commands: Commands,
    mut weapon: CmdWriter<Weapon>, mut stats: ResMut<Stats>, mut beats: ResMut<Beats>,
    mut time_mode: ResMut<TimeMode>, real_time: Res<Time>,
) {
    let (entity, pos, mut player, mut kctr) = match player.get_single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };
    let pos = pos.pos_2d();

    let mut mov = Vec2::ZERO;
    let mut dash = false;
    for action in input.iter() {
        match action {
            InputAction::MoveLeft => mov.x -= 1.,
            InputAction::MoveRight => mov.x += 1.,
            InputAction::MoveUp => mov.y += 1.,
            InputAction::MoveDown => mov.y -= 1.,

            InputAction::Dash => dash = true,

            InputAction::Fire => {
                if player.try_shoot(&real_time, false) {
                    weapon.send((
                        entity,
                        Weapon::PlayerGun {
                            dir: window.cursor - pos,
                        },
                    ))
                }
            }
            InputAction::FireMega => {
                if player.try_shoot(&real_time, true) {
                    weapon.send((
                        entity,
                        Weapon::PlayerCrafted {
                            dir: window.cursor - pos,
                        },
                    ))
                }
            }
            InputAction::ChangeWeapon => {
                let stats = &mut *stats;
                std::mem::swap(&mut stats.player.weapon0, &mut stats.player.weapon1);
                player.fire_lock = None;
            }
            InputAction::UberCharge => {
                if stats.ubercharge >= 1. {
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
            }

            _ => (),
        }
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
    mut input: EventReader<InputAction>, input_map: Res<InputMap>,
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
                    ui.label(input_map.map[InputAction::Respawn].0.to_string());
                    ui.visuals_mut().override_text_color = None;
                    ui.label("] to restart level");
                });
            },
        );
        for action in input.iter() {
            match action {
                InputAction::Respawn => spawn.despawn = Some(true),
                _ => (),
            }
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
        if !spawn.waiting_for_next_wave && beats.level == 0 {
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

    events.iter_cmd_mut(&mut player, |_, (_, _, mut player)| {
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
    mut commands: Commands, mut input: EventReader<InputAction>, input_map: Res<InputMap>,
    mut data: Local<NextWaveMenu>,
) {
    let input_action = InputAction::Respawn;

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
                (input_map.map[input_action].0.to_string(), Color::RED),
                ("] to go to next level".to_string(), Color::WHITE),
            ],
            Some(_) => vec![],
        };
        data.text = Some(
            commands
                .spawn_bundle(SpatialBundle::default())
                .insert(WorldText { text, size: 2. })
                .id(),
        );
    }
    // waiting for user input
    else if let Some(text) = data.text {
        if spawn.is_game_running() {
            for input in input.iter() {
                if *input == input_action {
                    if spawn.tutorial.is_none() {
                        stats.wave += 1;
                    }
                    spawn.despawn = Some(true);

                    commands.entity(text).despawn_recursive();
                    data.text = None;
                    break;
                }
            }
        } else {
            // npott anymore
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
                for v in [stats.player.weapon0, stats.player.weapon1] {
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

struct CraftMenu {
    slot0: CraftPart,
    slot1: CraftPart,
    show: bool,
}

impl Default for CraftMenu {
    fn default() -> Self {
        Self {
            slot0: CraftPart::Emitter,
            slot1: CraftPart::Laser,
            show: false,
        }
    }
}

fn craft_menu(
    mut ctx: ResMut<EguiContext>, mut stats: ResMut<Stats>, input_map: Res<InputMap>,
    mut input: EventReader<InputAction>, mut menu: Local<CraftMenu>, keys: Res<Input<KeyCode>>,
    mut time_mode: ResMut<TimeMode>, player: Query<(), With<Player>>,
) {
    time_mode.craft_menu = menu.show;
    time_mode.player_alive = !player.is_empty();

    if menu.show {
        if is_exit_menu(&keys) {
            menu.show = false
        }

        let craft_result = match (menu.slot0, menu.slot1) {
            (CraftPart::Generator, CraftPart::Laser) => Some(CraftedWeapon::Plasma),
            (CraftPart::Generator, CraftPart::Magnet) => Some(CraftedWeapon::Shield),
            (CraftPart::Emitter, CraftPart::Laser) => Some(CraftedWeapon::Railgun),
            (CraftPart::Emitter, CraftPart::Magnet) => Some(CraftedWeapon::Repeller),
            _ => {
                log::error!("Invalid craft slots: {:?} and {:?}", menu.slot0, menu.slot1);
                None
            }
        };

        ctx.popup(
            "player::craft_menu",
            vec2(0., -1.),
            true,
            egui::Order::Background,
            |ui| {
                ui.label("Press [ESC] or [M] to close this menu"); // TODO: unhardcode
                ui.group(|ui| {
                    ui.label("Available parts");
                    ui.group(|ui| {
                        ui.label("Press 1/2/3/4 to put part in the slot.");
                        ui.label("Currently there are only 4 combinations,");
                        ui.label("so slot is chosen automatically.");
                    });
                    for (key, value) in stats.player.craft_parts.iter() {
                        let (action, name, slot) = key.description();
                        ui.visuals_mut().override_text_color = Some(if *value == 0 {
                            egui::Color32::DARK_GRAY
                        } else {
                            egui::Color32::WHITE
                        });
                        ui.label(format!(
                            "[slot {}: {}] {} x{}",
                            slot,
                            input_map.map[action].0.to_string(),
                            name,
                            *value
                        ));
                        ui.visuals_mut().override_text_color = None;
                    }
                });
                ui.group(|ui| {
                    ui.visuals_mut().override_text_color =
                        Some(if stats.player.craft_parts[menu.slot0] == 0 {
                            egui::Color32::DARK_GRAY
                        } else {
                            egui::Color32::WHITE
                        });
                    ui.label(format!("Slot 1: {}", menu.slot0.description().1));

                    ui.visuals_mut().override_text_color =
                        Some(if stats.player.craft_parts[menu.slot1] == 0 {
                            egui::Color32::DARK_GRAY
                        } else {
                            egui::Color32::WHITE
                        });
                    ui.label(format!("Slot 2: {}", menu.slot1.description().1));
                    ui.visuals_mut().override_text_color = None;

                    if let Some(result) = craft_result {
                        let (name, text, _) = result.description();
                        ui.label(format!("Result: {}", name));
                        ui.label(text);
                    } else {
                        ui.label("CRAFT IS BROKEN LOL");
                        ui.label("TRY TO PUSH SOME BUTTONS");
                    }
                });
                ui.label("Press [C] to craft new weapon (replaces current)");
            },
        );

        for action in input.iter() {
            match action {
                InputAction::CraftSelect1 => menu.slot0 = CraftPart::Generator,
                InputAction::CraftSelect2 => menu.slot0 = CraftPart::Emitter,
                InputAction::CraftSelect3 => menu.slot1 = CraftPart::Laser,
                InputAction::CraftSelect4 => menu.slot1 = CraftPart::Magnet,
                InputAction::Craft => {
                    if let Some(weapon) = craft_result {
                        if stats.player.craft_parts[menu.slot0] != 0
                            && stats.player.craft_parts[menu.slot1] != 0
                        {
                            stats.player.craft_parts[menu.slot0] -= 1;
                            stats.player.craft_parts[menu.slot1] -= 1;

                            stats.player.weapon0 = Some((weapon, weapon.description().2))
                        }
                    }
                }
                _ => (),
            }
        }
    } else {
        for action in input.iter() {
            if *action == InputAction::Craft {
                menu.show = true
            }
        }
    }
}
