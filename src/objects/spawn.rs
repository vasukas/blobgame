use super::{player::Player, stats::Stats};
use crate::{
    common::*,
    mechanics::{
        ai::*,
        damage::{EnemyType, Team},
        health::{DieAfter, Health},
    },
    objects::{
        grid::GridBar,
        loot::{DropsLoot, Loot},
        stats::DeathPoints,
        weapon::Weapon,
    },
    present::{
        camera::WorldCamera,
        effect::{FlashOnDamage, SpawnEffect},
    },
    settings::Difficulty,
};
use serde::{Deserialize, Serialize};
use std::f32::consts::SQRT_2;

/// Object which must be despawned
#[derive(Component)]
pub struct GameplayObject;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum SpawnObject {
    Wall { extents: Vec2 },
    Turret(TurretType),
    Boss,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurretType {
    Simple,
    Advanced,
    Rotating,
}

//

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnControl>()
            .init_resource::<WaveData>()
            .init_resource::<TutorialText>()
            .init_resource::<LevelData>()
            .add_event::<LevelProgressEvent>()
            .add_system_to_stage(CoreStage::First, spawn.exclusive_system())
            .add_system(wave_end_detect)
            .add_system(draw_tutorial_text);
    }
}

#[derive(Default)]
struct WaveData {
    entities: Vec<Entity>,
}

#[derive(Component)]
struct TemporaryWall;

fn spawn(
    mut commands: Commands, mut control: ResMut<SpawnControl>,
    entities: Query<Entity, With<GameplayObject>>, mut camera: Query<&mut WorldCamera>,
    mut stats: ResMut<Stats>, mut wave_data: ResMut<WaveData>,
    mut wave_event: EventWriter<LevelProgressEvent>, settings: Res<Settings>,
    mut tutorial_text: ResMut<TutorialText>, tmp_walls: Query<Entity, With<TemporaryWall>>,
) {
    if let Some(respawn) = control.despawn.take() {
        // despawn all objects only if it's despawn or respawn, but not next if it's next wave
        let despawn = !respawn || control.wave_spawned == Some(stats.wave);
        if despawn {
            for entity in entities.iter() {
                commands.entity(entity).despawn_recursive()
            }
        } else {
            for entity in tmp_walls.iter() {
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

        let new = control.wave_spawned != Some(stats.wave);
        control.wave_spawned = Some(stats.wave);
        control.waiting_for_next_wave = false;
        *wave_data = default();
        wave_event.send(LevelProgressEvent::Started { new });

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

        /*
        match control.tutorial {
            // not tutorial, actual game
            None => {
                tutorial_text.0 = default();

                match stats.wave % 6 {
                    0 => {
                        for pos in [
                            vec2(world_size.x * -0.2, world_size.y * -0.3),
                            vec2(world_size.x * -0.2, world_size.y * 0.3),
                            //
                            vec2(world_size.x * 0.2, world_size.y * -0.3),
                            vec2(world_size.x * 0.2, world_size.y * 0.3),
                        ] {
                            create_wall(&mut commands, offset + pos, Vec2::splat(1.3))
                        }
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
                    }
                    1 => {
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(-12., -7.),
                            settings.difficulty,
                            TurretType::Simple,
                        ));
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(-12., 7.),
                            settings.difficulty,
                            TurretType::Simple,
                        ));
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(12., -7.),
                            settings.difficulty,
                            TurretType::Simple,
                        ));
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(12., 7.),
                            settings.difficulty,
                            TurretType::Simple,
                        ));
                    }
                    2 => {
                        for pos in [vec2(0., 8.)] {
                            create_wall(&mut commands, offset + pos, Vec2::splat(1.3))
                        }
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(-12., 7.),
                            settings.difficulty,
                            TurretType::Advanced,
                        ));
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(12., 7.),
                            settings.difficulty,
                            TurretType::Advanced,
                        ));
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(0., -1.),
                            settings.difficulty,
                            TurretType::Rotating,
                        ));
                    }
                    3 => {
                        for pos in [
                            vec2(world_size.x * -0.1, world_size.y * -0.2),
                            vec2(world_size.x * -0.1, world_size.y * 0.2),
                            vec2(world_size.x * -0.1, world_size.y * -0.4),
                            vec2(world_size.x * -0.1, world_size.y * 0.4),
                            //
                            vec2(world_size.x * 0.1, world_size.y * -0.2),
                            vec2(world_size.x * 0.1, world_size.y * 0.2),
                            vec2(world_size.x * 0.1, world_size.y * -0.4),
                            vec2(world_size.x * 0.1, world_size.y * 0.4),
                        ] {
                            create_wall(&mut commands, offset + pos, Vec2::splat(1.3))
                        }
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
                    4 => {
                        for pos in [
                            vec2(world_size.x * -0.36, -1.),
                            vec2(world_size.x * -0.23, 1.),
                            vec2(world_size.x * -0.1, 1.),
                            vec2(world_size.x * 0.1, -1.),
                            vec2(world_size.x * 0.23, 1.),
                            vec2(world_size.x * 0.36, -1.),
                        ] {
                            create_wall(&mut commands, offset + pos, Vec2::splat(1.))
                        }
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(-15., 5.),
                            settings.difficulty,
                            TurretType::Advanced,
                        ));
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(-15., -5.),
                            settings.difficulty,
                            TurretType::Simple,
                        ));
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(15., 5.),
                            settings.difficulty,
                            TurretType::Advanced,
                        ));
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(15., -5.),
                            settings.difficulty,
                            TurretType::Simple,
                        ));
                    }
                    _ => {
                        for pos in [
                            vec2(world_size.x * -0.1, world_size.y * -0.4),
                            vec2(world_size.x * -0.1, world_size.y * -0.2),
                            vec2(world_size.x * -0.1, world_size.y * 0.2),
                            //
                            vec2(world_size.x * 0.1, world_size.y * -0.4),
                            vec2(world_size.x * 0.1, world_size.y * -0.2),
                            vec2(world_size.x * 0.1, world_size.y * 0.2),
                        ] {
                            create_wall(&mut commands, offset + pos, Vec2::splat(1.3))
                        }
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(-10., 3.),
                            settings.difficulty,
                            TurretType::Rotating,
                        ));
                        wave_data.entities.push(create_turret(
                            &mut commands,
                            offset + vec2(10., -3.),
                            settings.difficulty,
                            TurretType::Advanced,
                        ));
                        wave_data.entities.push(
                            commands
                                .spawn_bundle(SpatialBundle::from_transform(Transform::new_2d(
                                    vec2(0., world_size.y / 2.),
                                )))
                                .insert(TheBoss { world_size, offset })
                                .insert(GameplayObject)
                                .id(),
                        );
                    }
                }
            }
        }
        */
    }
}

fn wave_end_detect(
    mut control: ResMut<SpawnControl>, entities: Query<()>, mut wave_data: ResMut<WaveData>,
    mut event: EventWriter<LevelProgressEvent>,
) {
    if control.is_game_running() {
        let was_empty = wave_data.entities.is_empty();
        wave_data.entities.retain(|e| entities.contains(*e));
        if wave_data.entities.is_empty() && !was_empty {
            control.waiting_for_next_level = true;
            event.send(LevelProgressEvent::Ended);
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
        .insert(Collider::cuboid(extents.x / 2., extents.y / 2.))
        .insert(TemporaryWall);
}

fn create_turret(
    commands: &mut Commands, origin: Vec2, _difficulty: Difficulty, ty: TurretType,
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
        .insert(EnemyType::Normal)
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
            TurretType::Rotating => 10.,
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
            vec![(Loot::Health(1.5), 0.5), (Loot::Weapon(None), 0.25)]
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
