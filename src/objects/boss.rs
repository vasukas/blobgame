use super::player::Player;
use crate::{
    common::*,
    mechanics::{
        damage::{BigProjectile, DamageOnContact, DamageRay, DieOnContact, Team},
        health::{Damage, DeathEvent, Health},
        physics::CollectContacts,
    },
    present::{
        effect::{
            ChargingAttack, DontSparkMe, Explosion, ExplosionPower, FlashOnDamage, RayEffect,
            SpawnEffect,
        },
        light::Light,
        sound::Sound,
    },
};

#[derive(Component)]
pub struct TheBoss {
    pub world_size: Vec2,
    pub offset: Vec2,
}

//

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(the_boss_spawn.exclusive_system())
            .add_system_to_stage(CoreStage::PostUpdate, boss_destruction.exclusive_system())
            .add_system(the_boss_logic)
            .add_system(update_guided_rocket);
    }
}

fn the_boss_spawn(
    mut commands: Commands, boss: Query<(Entity, &TheBoss), Added<TheBoss>>, time: Res<GameTime>,
) {
    use bevy_lyon::*;
    for (entity, boss) in boss.iter() {
        let center_radius = 1.;
        let tower_distance = 8.;
        let tower_width = 1.5;
        let tower_length = 2.;

        commands
            .entity(entity)
            .insert(BossState {
                parts: 3,
                die_start: None,
                count: -1,
            })
            .insert(BossLogic {
                start: time.now(),
                ray_min: boss.offset.x - boss.world_size.x / 2.,
                ray_max: boss.offset.x + boss.world_size.x / 2.,
                x_tower: tower_distance,
                ..default()
            })
            .with_children(|parent| {
                let outline_mode = StrokeMode::new(Color::rgb(0.8, 0.8, 0.8), 0.1);
                let health = 40.;

                // background
                parent
                    .spawn_bundle(GeometryBuilder::build_as(
                        &shapes::Polygon {
                            points: vec![
                                vec2(-(tower_distance + tower_width), 50.),
                                vec2(-(tower_distance + tower_width), -1.),
                                vec2(tower_distance + tower_width, -1.),
                                vec2(tower_distance + tower_width, 50.),
                            ],
                            closed: true,
                        },
                        DrawMode::Outlined {
                            fill_mode: FillMode::color(Color::rgb(0.2, 0., 0.)),
                            outline_mode,
                        },
                        default(),
                    ))
                    .insert(Depth::BossBackground);

                // center
                parent
                    .spawn_bundle(GeometryBuilder::build_as(
                        &shapes::Circle {
                            radius: center_radius,
                            center: Vec2::ZERO,
                        },
                        DrawMode::Outlined {
                            fill_mode: FillMode::color(Color::rgb(1., 0.3, 0.3)),
                            outline_mode,
                        },
                        Transform::new_2d(vec2(0., -center_radius * 0.2 - 1.)),
                    ))
                    .insert(Depth::Boss)
                    .insert(SpawnEffect {
                        radius: center_radius,
                    })
                    .insert(FlashOnDamage::Radius(center_radius))
                    //
                    .insert(Team::Enemy)
                    .insert(Health::new(health * 2.))
                    .insert(RigidBody::KinematicPositionBased)
                    .insert(PhysicsType::Solid.rapier())
                    .insert(Collider::ball(center_radius))
                    //
                    .insert(BossPart(entity, 0));

                // towers
                for (x, id) in [(-tower_distance, 1), (tower_distance, 2)] {
                    parent
                        .spawn_bundle(GeometryBuilder::build_as(
                            &shapes::Rectangle {
                                extents: vec2(tower_width, tower_length),
                                origin: RectangleOrigin::Center,
                            },
                            DrawMode::Outlined {
                                fill_mode: FillMode::color(Color::rgb(0.8, 0.8, 1.)),
                                outline_mode,
                            },
                            Transform::new_2d(vec2(x, -1.)),
                        ))
                        .insert(Depth::Boss)
                        .insert(SpawnEffect {
                            radius: tower_width.max(tower_length),
                        })
                        .insert(FlashOnDamage::Radius(tower_width.max(tower_length)))
                        //
                        .insert(Team::Enemy)
                        .insert(Health::new(health))
                        .insert(RigidBody::KinematicPositionBased)
                        .insert(PhysicsType::Solid.rapier())
                        .insert(Collider::cuboid(tower_width / 2., tower_length / 2.))
                        //
                        .insert(BossPart(entity, id));
                }
            });
    }
}

#[derive(Component)]
struct BossState {
    parts: i32,
    die_start: Option<Duration>,
    count: i32,
}

#[derive(Component)]
struct BossPart(Entity, u8);

fn boss_destruction(
    mut commands: Commands, mut death: CmdReader<DeathEvent>, mut parts: Query<&BossPart>,
    mut bosses: Query<(Entity, &mut BossState, &GlobalTransform)>, time: Res<GameTime>,
    mut explode: EventWriter<Explosion>,
) {
    let tower_distance = 6.;
    let tower_width = 1.5;

    death.iter_cmd_mut(&mut parts, |_, part| {
        if let Ok((_, mut boss, _)) = bosses.get_mut(part.0) {
            boss.parts -= 1;
            if boss.parts <= 0 {
                boss.die_start.get_or_insert(time.now());
            }
        }
    });

    for (entity, mut boss, pos) in bosses.iter_mut() {
        if let Some(start) = boss.die_start {
            let passed = time.passed(start);
            if passed > Duration::from_secs(3) {
                commands.entity(entity).despawn_recursive()
            } else {
                let new_count = (passed.as_micros() / Duration::from_millis(80).as_micros()) as i32;
                if new_count != boss.count {
                    boss.count = new_count;

                    use rand::*;
                    explode.send(Explosion {
                        origin: pos.pos_2d()
                            + vec2(
                                thread_rng().gen_range(-1. ..1.)
                                    * (tower_distance + tower_width + 3.),
                                thread_rng().gen_range(-3. ..1.),
                            ),
                        color0: Color::YELLOW,
                        color1: Color::RED,
                        time: Duration::from_millis(thread_rng().gen_range(400..700)),
                        radius: thread_rng().gen_range(2. ..5.),
                        power: ExplosionPower::Small,
                    });
                }
            }
        }
    }
}

#[derive(Component, Default)]
struct BossLogic {
    start: Duration,
    ray_min: f32,
    ray_max: f32,
    x_tower: f32,

    stage: usize,
    count: usize,
}

#[derive(Clone, Copy)]
enum BossStage {
    Wait,
    Sweep(usize),
    Rocket,
    Open,
}

fn the_boss_logic(
    mut commands: Commands,
    mut logic: Query<(&GlobalTransform, &mut Transform, &mut BossLogic, &TheBoss)>,
    time: Res<GameTime>, parts: Query<(Entity, &BossPart)>, mut sound: EventWriter<Sound>,
    assets: Res<MyAssets>,
) {
    let charge_duration = Duration::from_millis(1000);
    let wait_duration = Duration::from_millis(1500);
    let sweep_duration = Duration::from_millis(2000);
    let targeted_speed = 7.5;
    let rocket_count = 10;
    let rocket_wait = Duration::from_millis(750);
    let open_duration = Duration::from_millis(3500);
    let y_speed = 12.;

    let stages = [
        BossStage::Wait,
        BossStage::Sweep(0),
        BossStage::Sweep(1),
        BossStage::Sweep(2),
        BossStage::Sweep(3),
        BossStage::Wait,
        BossStage::Rocket,
        BossStage::Wait,
        BossStage::Open,
    ];
    let get_part = |id: u8| parts.iter().find(|v| v.1 .1 == id).map(|v| v.0);

    for (pos, mut transform, mut logic, bossinfo) in logic.iter_mut() {
        let pos = pos.pos_2d();
        let mut target_y = 0.;

        let duration = match stages[logic.stage] {
            BossStage::Wait => {
                target_y = 5.;
                wait_duration
            }
            BossStage::Sweep(which) => {
                if time.passed(logic.start) >= charge_duration && logic.count == 0 {
                    logic.count = 1;

                    let mut rays = vec![];
                    let speed = (logic.ray_max - logic.ray_min) / sweep_duration.as_secs_f32();
                    match which {
                        0 => rays.push((logic.ray_min, Ray::Speed(speed))),
                        1 => rays.push((logic.ray_max, Ray::Speed(-speed))),
                        2 => rays.push((logic.ray_min, Ray::Speed(speed))),
                        _ => rays.push((0., Ray::Player(targeted_speed))),
                    }
                    if which == 2 {
                        rays.push((logic.ray_max, Ray::Speed(-speed)))
                    }
                    for (pos, ray) in rays {
                        let fade_time = Duration::from_millis(250);
                        commands
                            .spawn_bundle(SpatialBundle::from_transform({
                                let mut t = Transform::new_2d(vec2(pos, transform.translation.y));
                                t.set_angle_2d(-TAU / 2.);
                                t
                            }))
                            .insert(GameplayObject)
                            .insert(Depth::ImportantEffect)
                            .insert(RayEffect {
                                color: Color::RED,
                                length: 50.,
                                width: 1.,
                                duration: sweep_duration - fade_time,
                                fade_time,
                                destroy_parent: true,
                                ..default()
                            })
                            .insert(ray)
                            .insert(Team::Enemy)
                            .insert(Damage::new(1.))
                            .insert(DamageRay {
                                explosion_effect: Some(Explosion {
                                    color0: Color::RED,
                                    color1: Color::RED,
                                    time: Duration::from_millis(200),
                                    radius: 0.9,
                                    ..default()
                                }),
                                ignore_obstacles: true,
                                ..default()
                            })
                            .insert(DontSparkMe);
                    }
                }
                sweep_duration + charge_duration
            }
            BossStage::Rocket => {
                if time.passed(logic.start) >= charge_duration {
                    let count = ((time.passed(logic.start) - charge_duration).as_micros()
                        / rocket_wait.as_micros()) as usize
                        + 1;
                    if count != logic.count {
                        logic.count = count;

                        if let Some(_) = get_part((logic.count % 2) as u8) {
                            let x =
                                if logic.count % 2 == 0 { logic.x_tower } else { -logic.x_tower };

                            use bevy_lyon::*;
                            let radius = 0.5;
                            commands
                                .spawn_bundle(GeometryBuilder::build_as(
                                    &shapes::Circle {
                                        radius,
                                        center: Vec2::ZERO,
                                    },
                                    DrawMode::Fill(FillMode::color(Color::WHITE)),
                                    {
                                        let mut t = Transform::new_2d(vec2(
                                            x,
                                            transform.translation.y - radius - 3.,
                                        ));
                                        t.set_angle_2d(-TAU / 2.);
                                        t
                                    },
                                ))
                                .insert(Depth::Projectile)
                                .insert(Light {
                                    radius: 2.,
                                    color: Color::WHITE.with_a(0.07),
                                })
                                .insert(
                                    Explosion {
                                        origin: Vec2::ZERO,
                                        color0: Color::GREEN,
                                        color1: Color::YELLOW,
                                        time: Duration::from_millis(400),
                                        radius: 0.5,
                                        power: ExplosionPower::Small,
                                    }
                                    .death(),
                                )
                                //
                                .insert(GameplayObject)
                                .insert(Damage::new(1.))
                                .insert(Team::Enemy)
                                .insert(DamageOnContact)
                                .insert(DieOnContact)
                                .insert(BigProjectile)
                                .insert(CollectContacts::default())
                                .insert(Health::new(2.))
                                //
                                .insert(RigidBody::Dynamic)
                                .insert(Collider::ball(radius))
                                .insert(ColliderMassProperties::Mass(3.))
                                .insert(PhysicsType::Projectile.rapier())
                                .insert(Velocity::linear(-Vec2::Y))
                                .insert(GuidedRocket {
                                    speed: 10.,
                                    accel: 12.,
                                });
                        }
                    }
                }
                rocket_count * rocket_wait + charge_duration
            }
            BossStage::Open => {
                target_y = -1.;
                open_duration
            }
        };
        if time.reached(logic.start + duration) {
            loop {
                logic.stage = (logic.stage + 1) % stages.len();
                match stages[logic.stage] {
                    BossStage::Wait | BossStage::Open => break,
                    BossStage::Sweep(_) => {
                        if let Some(entity) = get_part(0) {
                            commands.entity(entity).insert(ChargingAttack {
                                radius: 3.,
                                duration: charge_duration,
                                color: Color::rgb(1., 0.4, 0.3),
                            });
                            sound.send(Sound {
                                sound: assets.ray_charge.clone(),
                                position: Some(pos),
                                ..default()
                            });

                            logic.count = 0;
                            break;
                        }
                    }
                    BossStage::Rocket => {
                        let mut any = false;
                        for entity in [get_part(1), get_part(2)] {
                            if let Some(entity) = entity {
                                commands.entity(entity).insert(ChargingAttack {
                                    radius: 3.,
                                    duration: charge_duration,
                                    color: Color::rgb(0.8, 1., 1.),
                                });
                                sound.send(Sound {
                                    sound: assets.ray_charge.clone(),
                                    position: Some(pos),
                                    ..default()
                                });

                                any = true;
                            }
                        }
                        if any {
                            break;
                        }
                    }
                }
            }
            logic.start = time.now();
        }

        let target_y = bossinfo.world_size.y * 0.5 + target_y;
        let max_y_delta = y_speed * time.delta_seconds();
        let y_delta = (target_y - transform.translation.y).clamp(-max_y_delta, max_y_delta);
        transform.translation.y += y_delta;
    }
}

#[derive(Component)]
struct GuidedRocket {
    speed: f32,
    accel: f32,
}

#[derive(Component)]
enum Ray {
    Speed(f32),
    Player(f32),
}

fn update_guided_rocket(
    mut rockets: Query<(&GlobalTransform, &mut Velocity, &GuidedRocket)>,
    player: Query<&GlobalTransform, With<Player>>, time: Res<GameTime>,
    mut rays: Query<(&mut Transform, &Ray)>,
) {
    for (mut transform, ray) in rays.iter_mut() {
        match *ray {
            Ray::Speed(speed) => transform.translation.x += speed * time.delta_seconds(),
            _ => (),
        }
    }
    let target = match player.get_single() {
        Ok(pos) => pos.pos_2d(),
        Err(_) => return,
    };
    for (pos, mut velocity, rocket) in rockets.iter_mut() {
        let target = (target - pos.pos_2d()).clamp_length(0., rocket.speed);
        let delta = (target - velocity.linvel).clamp_length(0., rocket.accel);
        velocity.linvel += delta * time.delta_seconds();
    }
    for (mut transform, ray) in rays.iter_mut() {
        match *ray {
            Ray::Player(speed) => {
                let speed = speed * time.delta_seconds();
                let delta = (target.x - transform.translation.x).clamp(-speed, speed);
                transform.translation.x += delta
            }
            _ => (),
        }
    }
}
