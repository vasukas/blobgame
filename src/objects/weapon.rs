use crate::{
    common::*,
    mechanics::{
        damage::*,
        health::{Damage, DieAfter, Health},
        physics::CollectContacts,
    },
    objects::player::Player,
    present::{
        effect::{DontSparkMe, Explosion, ExplosionPower, RayEffect},
        light::Light,
    },
};

/// Command event
#[derive(Clone, Copy, Default)]
pub enum Weapon {
    #[default]
    None,
    Turret,

    PlayerGun {
        dir: Vec2,
    },
}

#[derive(SystemLabel)]
pub struct WeaponSystemLabel;

//

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<(Entity, Weapon)>()
            .add_system(weapon.label(WeaponSystemLabel));
    }
}

fn weapon(
    mut commands: Commands, mut weapon: CmdReader<Weapon>,
    mut source: Query<(Entity, &GlobalTransform, &Team)>,
) {
    use bevy_lyon::*;
    weapon.iter_cmd_mut(
        &mut source,
        |weapon, (_entity, transform, team)| match *weapon {
            Weapon::None => log::warn!("Shooting Weapon::None"),

            Weapon::Turret => {
                let radius = 0.25;
                let (transform, velocity) = make_origin(transform, 1.5, 10.);
                commands
                    .spawn_bundle(GeometryBuilder::build_as(
                        &shapes::Polygon {
                            points: vec![
                                vec2(0., radius * 2.),
                                vec2(0., radius).rotated(160f32.to_radians()),
                                vec2(0., radius).rotated(-160f32.to_radians()),
                            ],
                            closed: true,
                        },
                        DrawMode::Fill(FillMode::color(Color::rgb(1., 1., 0.6))),
                        transform,
                    ))
                    .insert(Depth::Projectile)
                    .insert(Light {
                        radius: 2.,
                        color: Color::rgb(1., 1., 0.8).with_a(0.07),
                    })
                    .insert(
                        Explosion {
                            origin: Vec2::ZERO,
                            color0: Color::YELLOW,
                            color1: Color::RED,
                            time: Duration::from_millis(400),
                            radius: 0.5,
                            power: ExplosionPower::Small,
                        }
                        .death(),
                    )
                    //
                    .insert(GameplayObject)
                    .insert(Damage::new(1.))
                    .insert(*team)
                    .insert(DamageOnContact)
                    .insert(DieOnContact)
                    .insert(CollectContacts::default())
                    .insert(Health::new(0.1))
                    //
                    .insert(RigidBody::Dynamic)
                    .insert(Collider::ball(radius))
                    .insert(ColliderMassProperties::Mass(1.))
                    .insert(PhysicsType::Projectile.rapier())
                    .insert(Velocity::linear(velocity));
            }

            Weapon::PlayerGun { dir } => {
                let mut transform = Transform::new_2d(
                    transform.pos_2d() + dir.normalize_or_zero() * Player::RADIUS,
                );
                transform.set_angle_2d(dir.angle());
                commands
                    .spawn_bundle(SpatialBundle::from_transform(transform))
                    .insert(GameplayObject)
                    .insert(Damage::new(1.))
                    .insert(*team)
                    .insert(DieAfter::one_frame())
                    .insert(DamageRay {
                        spawn_effect: Some(RayEffect {
                            color: Color::CYAN.with_a(0.4),
                            width: 0.4,
                            fade_time: Duration::from_millis(300),
                            ..default()
                        }),
                        ..default()
                    })
                    .insert(DontSparkMe);
            }
        },
    );
}

fn make_origin(pos: &GlobalTransform, distance: f32, speed: f32) -> (Transform, Vec2) {
    let mut pos: Transform = (*pos).into();
    let forward = pos.local_y().truncate();
    pos.add_2d(forward * distance);
    (pos, forward * speed)
}
