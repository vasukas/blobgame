use super::player::Player;
use crate::{
    common::*,
    mechanics::{ai::*, damage::Team},
    objects::weapon::Weapon,
    present::{camera::WorldCamera, effect::SpawnEffect},
};

/// Object which must be despawned
#[derive(Component)]
pub struct GameplayObject;

/// Resource
#[derive(Default)]
pub struct SpawnControl {
    /// Current state
    pub spawned: bool,

    /// Set this to Some(true) to respawn, to Some(false) to despawn
    pub despawn: Option<bool>,
}

//

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnControl>()
            .add_system_to_stage(CoreStage::First, spawn.exclusive_system());
    }
}

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

fn spawn(
    mut commands: Commands, mut control: ResMut<SpawnControl>,
    entities: Query<Entity, With<GameplayObject>>, mut camera: Query<&mut WorldCamera>,
) {
    if let Some(respawn) = control.despawn.take() {
        for entity in entities.iter() {
            commands.entity(entity).despawn_recursive()
        }
        control.spawned = respawn;
        if respawn {
            use bevy_lyon::*;

            let world_ratio = 16. / 9.;
            let world_size = vec2(40., 40. / world_ratio);
            camera.single_mut().target_size = world_size;

            // world border
            commands
                .spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Rectangle {
                        extents: world_size,
                        origin: RectangleOrigin::Center,
                    },
                    DrawMode::Stroke(StrokeMode::new(Color::WHITE * 0.3, 0.1)),
                    default(),
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
                create_wall(&mut commands, pos, Vec2::splat(1.5))
            }

            // the player
            commands
                .spawn_bundle(SpatialBundle::default())
                .insert(Player::default())
                .insert(GameplayObject)
                .insert(SpawnEffect { radius: 2. });

            // test turret
            let radius = 0.6;
            commands
                .spawn_bundle(GeometryBuilder::build_as(
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
                    // Transform::new_2d(-world_size / 3.),
                    Transform::new_2d(vec2(-5., 0.)),
                ))
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
                //
                .insert(RigidBody::Fixed)
                .insert(PhysicsType::Solid.rapier())
                .insert(Collider::ball(radius));
        }
    }
}
