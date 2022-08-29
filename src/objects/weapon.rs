use super::stats::Stats;
use crate::{
    common::*,
    mechanics::{
        damage::*,
        health::{Damage, DieAfter, Health},
        movement::KinematicController,
        physics::CollectContacts,
    },
    objects::player::Player,
    present::{
        effect::{DontSparkMe, Explosion, ExplosionPower, RayEffect},
        light::Light,
        sound::{Beats, Sound},
    },
};
use std::f32::consts::PI;

/// Command event
#[derive(Clone, Copy, Default)]
pub enum Weapon {
    #[default]
    None,
    Turret,

    PlayerGun {
        dir: Vec2,
    },
    PlayerCrafted {
        dir: Vec2,
    },
}

#[derive(Clone, Copy)]
pub enum CraftedWeapon {
    Plasma,
    Shield,
    Railgun,
    Repeller,
}

impl CraftedWeapon {
    // (name, description, max uses)
    pub fn description(&self) -> (&'static str, &'static str, f32) {
        match self {
            CraftedWeapon::Plasma => ("Plasma", "Shoots explosive energy balls", 10.),
            CraftedWeapon::Shield => (
                "Shield",
                "Generates force-field which absorbs incoming projectiles",
                10.,
            ),
            CraftedWeapon::Railgun => ("Railgun", "Shoots powerful piercing death ray", 10.),
            CraftedWeapon::Repeller => ("Repeller", "Pushes projectiles away from you", 15.),
        }
    }
}

//

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<(Entity, Weapon)>()
            .add_system_to_stage(CoreStage::PostUpdate, weapon);
    }
}

fn weapon(
    mut commands: Commands, mut weapon: CmdReader<Weapon>,
    mut source: Query<(
        Entity,
        &GlobalTransform,
        &Team,
        Option<&KinematicController>,
    )>,
    mut sound_cmd: EventWriter<Sound>, assets: Res<MyAssets>, beats: Res<Beats>,
    real_time: Res<Time>, mut stats: ResMut<Stats>,
) {
    use bevy_lyon::*;
    weapon.iter_cmd_mut(
        &mut source,
        |weapon, (_entity, transform, team, kinematic)| match *weapon {
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

                sound_cmd.send(Sound {
                    sound: assets.wpn_smg.clone(),
                    position: Some(transform.pos_2d()),
                });
            }

            Weapon::PlayerGun { dir } | Weapon::PlayerCrafted { dir } => {
                let mut transform = Transform::new_2d(
                    transform.pos_2d() + dir.normalize_or_zero() * Player::RADIUS,
                );
                let angle = dir.angle();
                transform.set_angle_2d(angle);

                let ultra_powered = beats.in_beat(&real_time);
                let powered = ultra_powered
                    || angle_delta(
                        angle,
                        kinematic
                            .and_then(|ctr| ctr.dash.map(|v| v.0.angle()))
                            .unwrap_or(angle + PI),
                    )
                    .abs()
                        < 60f32.to_radians();

                let mut commands = commands.spawn_bundle(SpatialBundle::from_transform(transform));
                commands.insert(GameplayObject).insert(*team);

                let (damage, ray, sound) = match weapon {
                    Weapon::PlayerGun { .. } => (
                        [1., 3., 6.],
                        Some(DamageRay {
                            spawn_effect: Some(RayEffect {
                                color: if powered || ultra_powered {
                                    Color::ORANGE_RED
                                } else {
                                    Color::CYAN
                                }
                                .with_a(0.3),
                                width: 0.4,
                                fade_time: Duration::from_millis(if powered { 400 } else { 300 }),
                                ..default()
                            }),
                            explosion_effect: Some(Explosion {
                                color0: Color::WHITE,
                                color1: Color::WHITE,
                                time: Duration::from_millis(300),
                                radius: 1.,
                                ..default()
                            }),
                            ..default()
                        }),
                        if powered {
                            assets.player_gun_powered.clone()
                        } else {
                            assets.player_gun.clone()
                        },
                    ),
                    _ => {
                        let mega_weapon = match stats.player.weapon0.as_mut() {
                            Some(w) => w,
                            None => {
                                commands.insert(DieAfter::one_frame());
                                return;
                            }
                        };
                        match mega_weapon {
                            (CraftedWeapon::Railgun, uses) => {
                                *uses -= 1.;
                                if *uses <= 0. {
                                    stats.player.weapon0 = None
                                }

                                (
                                    [4., 6., 12.],
                                    Some(DamageRay {
                                        spawn_effect: Some(RayEffect {
                                            color: if powered || ultra_powered {
                                                Color::WHITE
                                            } else {
                                                Color::ALICE_BLUE
                                            }
                                            .with_a(0.5),
                                            width: 0.4,
                                            fade_time: Duration::from_millis(if powered {
                                                400
                                            } else {
                                                300
                                            }),
                                            ..default()
                                        }),
                                        explosion_effect: Some(Explosion {
                                            color0: Color::WHITE,
                                            color1: Color::WHITE,
                                            time: Duration::from_millis(400),
                                            radius: 1.5,
                                            ..default()
                                        }),
                                        ..default()
                                    }),
                                    assets.player_railgun.clone(),
                                )
                            }
                            _ => todo!(),
                        }
                    }
                };

                commands.insert(Damage::new(
                    damage[if ultra_powered && powered {
                        2
                    } else if powered || ultra_powered {
                        1
                    } else {
                        0
                    }],
                ));

                if let Some(ray) = ray {
                    commands
                        .insert(ray)
                        .insert(DieAfter::one_frame())
                        .insert(DontSparkMe);
                }

                sound_cmd.send(Sound {
                    sound,
                    position: Some(transform.pos_2d()),
                });
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
