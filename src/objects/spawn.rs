use super::{player::Player, stats::Stats};
use crate::{
    common::*,
    mechanics::{
        ai::*,
        damage::Team,
        health::{DieAfter, Health},
    },
    objects::{
        grid::GridBar,
        loot::{CraftPart, DropsLoot, Loot},
        stats::DeathPoints,
        weapon::Weapon,
    },
    present::{
        camera::WorldCamera,
        effect::{FlashOnDamage, SpawnEffect},
    },
    settings::Difficulty,
};
use std::f32::consts::SQRT_2;

/// Object which must be despawned
#[derive(Component)]
pub struct GameplayObject;

/// Resource
#[derive(Default)]
pub struct SpawnControl {
    /// Current state
    pub wave_spawned: Option<usize>,
    pub waiting_for_next_wave: bool,

    /// Set this to Some(true) to respawn, to Some(false) to despawn
    pub despawn: Option<bool>,
    pub tutorial: Option<usize>,
}

impl SpawnControl {
    pub fn is_game_running(&self) -> bool {
        self.wave_spawned.is_some()
    }
}

/// Event, sent from here
#[derive(PartialEq, Eq)]
pub enum WaveEvent {
    Started,
    Ended,
    Restart, // sent with Started
}

//

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnControl>()
            .init_resource::<WaveData>()
            .init_resource::<TutorialText>()
            .add_event::<WaveEvent>()
            .add_system_to_stage(CoreStage::First, spawn.exclusive_system())
            .add_system(wave_end_detect)
            .add_system(draw_tutorial_text);
    }
}

#[derive(Default)]
struct WaveData {
    entities: Vec<Entity>,
}

fn spawn(
    mut commands: Commands, mut control: ResMut<SpawnControl>,
    entities: Query<Entity, With<GameplayObject>>, mut camera: Query<&mut WorldCamera>,
    mut stats: ResMut<Stats>, mut wave_data: ResMut<WaveData>,
    mut wave_event: EventWriter<WaveEvent>, settings: Res<Settings>,
    mut tutorial_text: ResMut<TutorialText>,
) {
    if let Some(respawn) = control.despawn.take() {
        // despawn all objects only if it's despawn or respawn, but not next if it's next wave
        let despawn = !respawn || control.wave_spawned == Some(stats.wave);
        if despawn {
            for entity in entities.iter() {
                commands.entity(entity).despawn_recursive()
            }
        }

        if !respawn {
            control.wave_spawned = None;
            return;
        }

        // first spawn ever
        let first_spawn = !control.is_game_running();
        if first_spawn {
            *stats = default();
        }

        if control.wave_spawned == Some(stats.wave) {
            wave_event.send(WaveEvent::Restart)
        }
        control.wave_spawned = Some(stats.wave);
        control.waiting_for_next_wave = false;
        *wave_data = default();
        wave_event.send(WaveEvent::Started);

        //

        use bevy_lyon::*;

        let world_ratio = 16. / 9.;
        let world_size = vec2(40., 40. / world_ratio);
        camera.single_mut().target_size = world_size + 0.1;

        let offset = vec2(1.8, 0.);
        let world_size = world_size - offset.abs() * 2.;

        // only on first spawn or respawn
        if first_spawn || despawn {
            // world border
            commands
                .spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Rectangle {
                        extents: world_size,
                        origin: RectangleOrigin::Center,
                    },
                    DrawMode::Stroke(StrokeMode::new(Color::WHITE * 0.3, 0.1)),
                    Transform::new_2d(offset),
                ))
                .insert(GameplayObject)
                .insert(Depth::Wall)
                //
                .insert(RigidBody::Fixed)
                .insert(PhysicsType::Solid.rapier())
                .insert(Collider::polyline(
                    vec![
                        vec2(-world_size.x / 2., -world_size.y / 2.),
                        vec2(world_size.x / 2., -world_size.y / 2.),
                        vec2(world_size.x / 2., world_size.y / 2.),
                        vec2(-world_size.x / 2., world_size.y / 2.),
                        vec2(-world_size.x / 2., -world_size.y / 2.),
                    ],
                    None,
                ));

            // background grid
            let cell_size = Player::DASH_DISTANCE / 2.;
            for i in 0..10000 {
                let x = i as f32 * cell_size;
                if x >= world_size.x / 2. {
                    break;
                }
                for x in [-x, x] {
                    commands
                        .spawn_bundle(GeometryBuilder::build_as(
                            &shapes::Line(vec2(x, -world_size.y / 2.), vec2(x, world_size.y / 2.)),
                            DrawMode::Stroke(StrokeMode::color(Color::NONE)),
                            Transform::new_2d(offset),
                        ))
                        .insert(GameplayObject)
                        .insert(Depth::BackgroundGrid)
                        .insert(GridBar {
                            coord: x / world_size.x * 2.,
                            vertical: true,
                        });
                    if x == 0. {
                        break;
                    }
                }
            }
            for i in 0..10000 {
                let y = i as f32 * cell_size;
                if y >= world_size.y / 2. {
                    break;
                }
                for y in [-y, y] {
                    commands
                        .spawn_bundle(GeometryBuilder::build_as(
                            &shapes::Line(vec2(-world_size.x / 2., y), vec2(world_size.x / 2., y)),
                            DrawMode::Stroke(StrokeMode::color(Color::NONE)),
                            Transform::new_2d(offset),
                        ))
                        .insert(GameplayObject)
                        .insert(Depth::BackgroundGrid)
                        .insert(GridBar {
                            coord: y / world_size.y * 2.,
                            vertical: false,
                        });
                    if y == 0. {
                        break;
                    }
                }
            }

            // the player
            commands
                .spawn_bundle(SpatialBundle::from_transform(Transform::new_2d(offset)))
                .insert(Player::default())
                .insert(GameplayObject)
                .insert(SpawnEffect { radius: 2. });
        }

        // empty entity for empty waves - so wave end detection will work
        wave_data
            .entities
            .push(commands.spawn().insert(DieAfter::one_frame()).id());

        // wave-specific spawns

        match control.tutorial {
            // IF YOU CHANGE ANYTHING HERE DON'T FORGET TO UPDATE HACK IN PLAYER NEXT WAVE MESSAGE!!!!
            Some(0) => {
                tutorial_text.0 = concat!(
                    "This is a tutorial message!\n",
                    "Press R key to show next message",
                );
            }
            Some(1) => {
                tutorial_text.0 = concat!(
                    "Move around with W/A/S/D keys.\n",
                    "Press SPACE key to dash in movement direction.\n",
                    "You can change direction mid-dash, but can't stop.\n",
                    "Dash gives temporary invincibilty, but consumes stamina.",
                    "\n\nPress R key to show next message",
                );
            }
            Some(2) => {
                tutorial_text.0 = concat!(
                    "Shoot with left mouse button.\n",
                    "Shoot in the movement direction just after starting dash\n",
                    "  to deal increased damage (ray will turn red).\n",
                    "Level is completed when all enemies are destroyed.",
                    "\n\nPress R key to show next message",
                );
            }
            Some(3) => {
                tutorial_text.0 = concat!("Destroy both turrets to finish the level!",);
                wave_data.entities.push(create_turret(
                    &mut commands,
                    offset + vec2(world_size.x * -0.4, world_size.y * 0.1),
                    settings.difficulty,
                    TurretType::Simple,
                ));
                wave_data.entities.push(create_turret(
                    &mut commands,
                    offset + vec2(world_size.x * 0.4, world_size.y * -0.1),
                    settings.difficulty,
                    TurretType::Simple,
                ));
            }
            Some(4) => {
                tutorial_text.0 = concat!(
                    "Sometimes enemies drop usable pieces.\n",
                    "Green ones restore your health.\n",
                    "Red ones can be used to craft additional weapons.\n",
                    "Only two such weapons can be equipped at one time, and they have limited uses.\n",
                    "Press C to access crafting menu.\n",
                    "Shoot with right mouse button; some attacks can be combined,\n",
                    "for example shooting plasma ball with railgun.",
                    "\n\nPress R key to show next message",
                );
            }
            Some(5) => {
                tutorial_text.0 = concat!(
                    "Over time you acquire focus charge.\n",
                    "Destroying enemies increases it faster, receiving damage decreases it.\n",
                    "When you have 100% charge, press SHIFT to enter focus mode.\n",
                    "If you shoot in sync with beat, damage is greatly increased.\n",
                    "  BUG: sound might get out of sync, but grid pulsation should be fine!\n",
                    "Time is limited, and receiving damage depletes it faster.",
                    "\n\nPress R key to show next message",
                );
            }
            Some(6) => {
                tutorial_text.0 = concat!("Try destroying the turret using focus mode!");
                wave_data.entities.push(create_turret(
                    &mut commands,
                    offset + vec2(0., world_size.y * 0.35),
                    settings.difficulty,
                    TurretType::Simple,
                ));
            }
            Some(_) => {
                tutorial_text.0 = concat!(
                    "That's it, end of tutorial!\n",
                    "Game currently has no ending,\n",
                    "Waves will be repeated after some time",
                    "\n\nPress R key to PLAY\n",
                    "And remember that you are damaged by your own explosions!",
                );
                control.tutorial = None;
            }

            // not tutorial, actual game
            None => {
                // TODO: warn about last wave
                tutorial_text.0 = default();
                control.tutorial = None;

                // static walls
                for pos in [
                    vec2(world_size.x * -0.1, world_size.y * -0.4),
                    vec2(world_size.x * -0.1, world_size.y * -0.2),
                    vec2(world_size.x * -0.1, world_size.y * 0.2),
                    vec2(world_size.x * -0.1, world_size.y * 0.4),
                    //
                    vec2(world_size.x * 0.1, world_size.y * -0.4),
                    vec2(world_size.x * 0.1, world_size.y * -0.2),
                    vec2(world_size.x * 0.1, world_size.y * 0.2),
                    vec2(world_size.x * 0.1, world_size.y * 0.4),
                ] {
                    create_wall(&mut commands, offset + pos, Vec2::splat(1.3))
                }

                // test turret
                if stats.wave % 2 == 1 {
                    wave_data.entities.push(create_turret(
                        &mut commands,
                        offset + vec2(-15., 0.),
                        settings.difficulty,
                        TurretType::Simple,
                    ));
                    wave_data.entities.push(create_turret(
                        &mut commands,
                        offset + vec2(15., 0.),
                        settings.difficulty,
                        TurretType::Simple,
                    ));
                } else {
                    wave_data.entities.push(create_turret(
                        &mut commands,
                        offset + vec2(-10., 10.),
                        settings.difficulty,
                        TurretType::Simple,
                    ));
                    wave_data.entities.push(create_turret(
                        &mut commands,
                        offset + vec2(-10., -10.),
                        settings.difficulty,
                        TurretType::Simple,
                    ));
                    wave_data.entities.push(create_turret(
                        &mut commands,
                        offset + vec2(12., 8.),
                        settings.difficulty,
                        TurretType::Advanced,
                    ));
                    wave_data.entities.push(create_turret(
                        &mut commands,
                        offset + vec2(10., -3.),
                        settings.difficulty,
                        TurretType::Rotating,
                    ));
                }
            }
        }
    }
}

fn wave_end_detect(
    mut control: ResMut<SpawnControl>, entities: Query<()>, mut wave_data: ResMut<WaveData>,
    mut event: EventWriter<WaveEvent>,
) {
    if control.is_game_running() {
        let was_empty = wave_data.entities.is_empty();
        wave_data.entities.retain(|e| entities.contains(*e));
        if wave_data.entities.is_empty() && !was_empty {
            control.waiting_for_next_wave = true;
            event.send(WaveEvent::Ended);

            if let Some(wave) = control.tutorial.as_mut() {
                *wave += 1
            }
        }
    }
}

//

fn create_wall(commands: &mut Commands, origin: Vec2, extents: Vec2) {
    use bevy_lyon::*;
    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shapes::Rectangle {
                extents,
                origin: RectangleOrigin::Center,
            },
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::BLACK),
                outline_mode: StrokeMode::new(Color::WHITE * 0.5, 0.1),
            },
            Transform::new_2d(origin),
        ))
        .insert(GameplayObject)
        .insert(Depth::Wall)
        .insert(SpawnEffect {
            radius: extents.max_element() * SQRT_2,
        })
        //
        .insert(RigidBody::Fixed)
        .insert(PhysicsType::Solid.rapier())
        .insert(Collider::cuboid(extents.x / 2., extents.y / 2.));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TurretType {
    Simple,
    Advanced,
    Rotating,
}

fn create_turret(
    commands: &mut Commands, origin: Vec2, difficulty: Difficulty, ty: TurretType,
) -> Entity {
    use bevy_lyon::*;

    let radius = 0.6;
    let mut commands = commands.spawn_bundle(GeometryBuilder::build_as(
        &shapes::Polygon {
            points: vec![
                vec2(0., radius),
                vec2(0., radius).rotated(150f32.to_radians()),
                vec2(0., radius).rotated(-150f32.to_radians()),
            ],
            closed: true,
        },
        match ty {
            TurretType::Simple => DrawMode::Outlined {
                fill_mode: FillMode::color(Color::ORANGE),
                outline_mode: StrokeMode::new(Color::YELLOW, 0.05),
            },
            TurretType::Advanced => DrawMode::Outlined {
                fill_mode: FillMode::color(Color::SEA_GREEN),
                outline_mode: StrokeMode::new(Color::DARK_GRAY, 0.05),
            },
            TurretType::Rotating => DrawMode::Outlined {
                fill_mode: FillMode::color(Color::CRIMSON),
                outline_mode: StrokeMode::new(Color::BEIGE, 0.05),
            },
        },
        Transform::new_2d(origin),
    ));
    match ty {
        TurretType::Simple | TurretType::Advanced => {
            commands.insert(LosCheck::default()).insert(FaceTarget {
                rotation_speed: TAU * 0.4,
                ..default()
            });
        }
        TurretType::Rotating => {
            commands.insert(HeSpinsHeRotats::new(TAU * 0.33));
        }
    }
    commands
        .insert(Depth::Player)
        .insert(SpawnEffect { radius: 1. })
        .insert(FlashOnDamage::Radius(radius))
        //
        .insert(GameplayObject)
        .insert(Target::Player)
        .insert(Team::Enemy)
        .insert(match ty {
            TurretType::Simple => AttackPattern::default()
                .stage(1, Duration::from_secs(1), AttackStage::Wait)
                .stage(
                    5,
                    Duration::from_millis(300),
                    AttackStage::Shoot(vec![Weapon::Turret]),
                )
                .stage(1, Duration::from_secs(1), AttackStage::Wait),

            TurretType::Advanced => AttackPattern::default()
                .stage(1, Duration::from_millis(500), AttackStage::Wait)
                .stage(
                    4,
                    Duration::from_millis(250),
                    AttackStage::Shoot(vec![
                        Weapon::AdvancedTurret { offset: 0. },
                        Weapon::AdvancedTurret { offset: -1.25 },
                        Weapon::AdvancedTurret { offset: 1.25 },
                    ]),
                ),

            TurretType::Rotating => AttackPattern::default().stage(
                1,
                Duration::from_millis(150),
                AttackStage::Shoot(vec![Weapon::RotatingTurret]),
            ),
        })
        .insert(Health::new(match ty {
            TurretType::Simple => 3.,
            TurretType::Advanced => 6.,
            TurretType::Rotating => 4.,
        }))
        .insert(match ty {
            TurretType::Simple => DeathPoints {
                value: 20,
                charge: 0.2,
            },
            TurretType::Advanced => DeathPoints {
                value: 40,
                charge: 0.3,
            },
            TurretType::Rotating => DeathPoints {
                value: 60,
                charge: 0.4,
            },
        })
        //
        .insert(RigidBody::Fixed)
        .insert(PhysicsType::Solid.rapier())
        .insert(Collider::ball(radius))
        .insert(DropsLoot({
            use rand::*;
            let mut loot = vec![];
            if match difficulty {
                Difficulty::Easy => true,
                Difficulty::Hard => thread_rng().gen_bool(0.8),
            } {
                if thread_rng().gen_bool(0.66) {
                    loot.push(Loot::Health {
                        value: match difficulty {
                            Difficulty::Easy => 4.,
                            Difficulty::Hard => 1.5,
                        },
                    });
                }
                if thread_rng().gen_bool(0.33) {
                    loot.push(Loot::CraftPart(CraftPart::random()));
                }
            }
            loot
        }));

    commands.id()
}

#[derive(Default)]
struct TutorialText(&'static str);

fn draw_tutorial_text(mut ctx: ResMut<EguiContext>, text: Res<TutorialText>) {
    ctx.popup(
        "draw_tutorial_text",
        vec2(0., 1.),
        false,
        egui::Order::Background,
        |ui| {
            let t = 1.;
            ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(
                lerp(128., 255., t) as u8,
                lerp(128., 255., t) as u8,
                lerp(128., 255., t) as u8,
            ));
            ui.heading(text.0);
        },
    );
}
