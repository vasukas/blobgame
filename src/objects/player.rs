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
        damage::Team,
        health::{Health, ReceivedDamage},
        movement::*,
    },
    present::{
        camera::WindowInfo,
        effect::Flash,
        hud_elements::WorldText,
        simple_sprite::SimpleSprite,
        sound::{AudioListener, Sound},
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
            .add_system(craft_menu);
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
            .insert(Collider::ball(radius * 0.66))
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
            .insert(Health::new(3.).armor())
            .insert(LootPicker {
                radius: Player::RADIUS,
                ..default()
            });
    }
}

fn controls(
    mut player: Query<(Entity, &GlobalTransform, &mut Player)>,
    mut input: EventReader<InputAction>, mut kinematic: CmdWriter<KinematicCommand>,
    window: Res<WindowInfo>, time: Res<GameTime>, mut commands: Commands,
    mut weapon: CmdWriter<Weapon>, mut stats: ResMut<Stats>,
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
            InputAction::FireMega => weapon.send((
                entity,
                Weapon::PlayerCrafted {
                    dir: window.cursor - pos,
                },
            )),
            InputAction::ChangeWeapon => {
                let stats = &mut *stats;
                std::mem::swap(&mut stats.player.weapon0, &mut stats.player.weapon1)
            }
            InputAction::UberCharge => {
                // TODO: implement
            }

            _ => (),
        }
    }
    if let Some(dir) = mov.try_normalize() {
        player.prev_move = dir;
        kinematic.send((entity, KinematicCommand::Move { dir }))
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

fn player_damage_reaction(
    mut commands: Commands, mut player: Query<(Entity, &Health), With<Player>>,
    mut events: CmdReader<ReceivedDamage>, mut was_damaged: Local<bool>,
    mut sound: EventWriter<Sound>, assets: Res<MyAssets>,
) {
    events.iter_cmd_mut(&mut player, |_, (entity, _)| {
        commands.entity(entity).insert(Flash {
            radius: Player::RADIUS,
            duration: Duration::from_millis(500),
            color0: Color::RED,
            color1: Color::NONE,
        });
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
        data.text = Some(
            commands
                .spawn_bundle(SpatialBundle::default())
                .insert(WorldText {
                    text: vec![
                        ("Press [".to_string(), Color::WHITE),
                        (input_map.map[input_action].0.to_string(), Color::RED),
                        ("] for next wave".to_string(), Color::WHITE),
                    ],
                    size: 2.,
                })
                .id(),
        );
    }
    // waiting for user input
    else if let Some(text) = data.text {
        if spawn.is_game_running() {
            for input in input.iter() {
                if *input == input_action {
                    stats.wave += 1;
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

fn hud_panel(mut ctx: ResMut<EguiContext>, stats: Res<Stats>, player: Query<(&Health, &Player)>) {
    ctx.popup(
        "player::hud_panel",
        vec2(-1., -1.),
        false,
        egui::Order::Background,
        |ui| {
            ui.label("WAVE");
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

                // TODO: show crafted weapons state
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
                            menu.show = false;

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
