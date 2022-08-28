use super::{player::Player, stats::Stats};
use crate::{
    common::*,
    mechanics::{ai::*, damage::Team, health::Health},
    objects::{
        grid::GridBar,
        loot::{CraftPart, Loot},
        stats::DeathPoints,
        weapon::Weapon,
    },
    present::{camera::WorldCamera, effect::SpawnEffect},
};

/// Object which must be despawned
#[derive(Component)]
pub struct GameplayObject;

/// Resource
#[derive(Default)]
pub struct SpawnControl {
    /// Current state
    pub wave_spawned: Option<usize>,

    /// Set this to Some(true) to respawn, to Some(false) to despawn
    pub despawn: Option<bool>,
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
            .add_event::<WaveEvent>()
            .add_system_to_stage(CoreStage::First, spawn.exclusive_system())
            .add_system(wave_end_detect);
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
    mut wave_event: EventWriter<WaveEvent>,
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

        // wave-specific spawns

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
            create_wall(&mut commands, offset + pos, Vec2::splat(1.5))
        }

        // test turret
        if stats.wave % 2 == 0 {
            wave_data
                .entities
                .push(create_turret(&mut commands, offset + vec2(-15., 0.)));
            wave_data
                .entities
                .push(create_turret(&mut commands, offset + vec2(15., 0.)));
        } else {
            wave_data
                .entities
                .push(create_turret(&mut commands, offset + vec2(-10., 10.)));
            wave_data
                .entities
                .push(create_turret(&mut commands, offset + vec2(10., 10.)));
            wave_data
                .entities
                .push(create_turret(&mut commands, offset + vec2(-10., -10.)));
            wave_data
                .entities
                .push(create_turret(&mut commands, offset + vec2(10., -10.)));
        }
    }
}

fn wave_end_detect(
    control: Res<SpawnControl>, entities: Query<()>, mut wave_data: ResMut<WaveData>,
    mut event: EventWriter<WaveEvent>,
) {
    if control.is_game_running() {
        let was_empty = wave_data.entities.is_empty();
        wave_data.entities.retain(|e| entities.contains(*e));
        if wave_data.entities.is_empty() && !was_empty {
            event.send(WaveEvent::Ended)
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
        //
        .insert(RigidBody::Fixed)
        .insert(PhysicsType::Solid.rapier())
        .insert(Collider::cuboid(extents.x / 2., extents.y / 2.));
}

fn create_turret(commands: &mut Commands, origin: Vec2) -> Entity {
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
        DrawMode::Outlined {
            fill_mode: FillMode::color(Color::ORANGE),
            outline_mode: StrokeMode::new(Color::YELLOW, 0.05),
        },
        Transform::new_2d(origin),
    ));
    commands
        .insert(Depth::Player)
        .insert(SpawnEffect { radius: 1. })
        //
        .insert(GameplayObject)
        .insert(Target::Player)
        .insert(Team::Enemy)
        .insert(LosCheck::default())
        .insert(FaceTarget {
            rotation_speed: TAU * 0.4,
            ..default()
        })
        .insert(
            AttackPattern::default()
                .stage(1, Duration::from_secs(1), AttackStage::Wait)
                .stage(
                    5,
                    Duration::from_millis(300),
                    AttackStage::Shoot(Weapon::Turret),
                )
                .stage(1, Duration::from_secs(1), AttackStage::Wait),
        )
        .insert(Health::new(3.))
        .insert(DeathPoints(20))
        //
        .insert(RigidBody::Fixed)
        .insert(PhysicsType::Solid.rapier())
        .insert(Collider::ball(radius));

    use rand::*;
    if thread_rng().gen_bool(0.7) {
        if thread_rng().gen_bool(0.6) {
            commands.insert(Loot::Health { value: 2. });
        } else {
            commands.insert(Loot::CraftPart(CraftPart::random()));
        }
    }

    commands.id()
}
