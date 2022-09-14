use bevy_prototype_lyon::prelude::DrawMode;

use super::{light::Light, sound::Sound};
use crate::{
    common::*,
    mechanics::health::{Damage, DeathEvent, Health, ReceivedDamage},
};

// Event
#[derive(Clone, Copy, Default)]
pub struct Explosion {
    pub origin: Vec2,
    pub color0: Color, // alpha ignored for colors
    pub color1: Color,
    pub time: Duration,
    pub radius: f32,
    pub power: ExplosionPower,
}

#[derive(Clone, Copy, Default)]
pub enum ExplosionPower {
    #[default]
    None,
    Small,
    Big,
}

impl Explosion {
    pub fn death(self) -> DeathExplosion {
        DeathExplosion(self)
    }
}

/// Show explosion on death
#[derive(Component)]
pub struct DeathExplosion(pub Explosion);

/// Show spawn effect once after creation
#[derive(Component)]
pub struct SpawnEffect {
    pub radius: f32,
}

/// Colored flash on an entity, once.
/// Gradually changes color during duration.
#[derive(Component)]
pub struct Flash {
    pub radius: f32,
    pub duration: Duration,
    pub color0: Color,
    pub color1: Color,
}

/// Event
pub struct HitSparks {
    pub origin: Vec2,
    pub damage: f32,
}

/// Don't generate on damage / on death sparks
#[derive(Component)]
pub struct DontSparkMe;

#[derive(Component, Clone, Copy, Default)]
pub struct RayEffect {
    pub color: Color,
    pub length: f32,
    pub width: f32,
    pub duration: Duration,
    pub fade_time: Duration,
    pub destroy_parent: bool,
}

#[derive(Component)]
pub enum FlashOnDamage {
    Radius(f32),
}

#[derive(Component, Clone, Copy)]
pub struct ChargingAttack {
    pub radius: f32,
    pub duration: Duration,
    pub color: Color,
}

//

pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Explosion>()
            .add_event::<HitSparks>()
            .add_system(explosion.exclusive_system())
            .add_system_to_stage(CoreStage::PostUpdate, death_explosion)
            .add_system_to_stage(CoreStage::Last, spawn_effect)
            .add_system_to_stage(CoreStage::PostUpdate, spawn_flash)
            .add_system(update_flash.exclusive_system())
            .add_system(hit_sparks)
            .add_system_to_stage(CoreStage::PostUpdate, hit_sparks_on_damage)
            .add_system(ray.exclusive_system())
            .add_system(flash_on_damage)
            .add_system(charging_attack.exclusive_system());
    }
}

#[derive(Component)]
struct ExplosionState {
    e: Explosion,
    start: Duration,
}

#[derive(Component)]
struct TemporaryHack;

fn explosion(
    mut commands: Commands, mut events: EventReader<Explosion>,
    mut explosions: Query<(Entity, &ExplosionState, &mut Light)>, time: Res<GameTime>,
    hack: Query<Entity, With<TemporaryHack>>, mut sounds: EventWriter<Sound>,
    assets: Res<MyAssets>,
) {
    let scale = 3.; // TODO: this is a hack to force lyon draw circles with more points

    for event in events.iter() {
        commands
            .spawn_bundle(SpatialBundle::from_transform({
                let mut t = Transform::new_2d(event.origin);
                t.scale = Vec3::new(1. / scale, 1. / scale, 1.);
                t
            }))
            .insert(GameplayObject)
            .insert(Depth::Effect)
            .insert(ExplosionState {
                e: *event,
                start: time.now(),
            })
            .insert(Light {
                radius: 0.,
                color: (event.color0 + event.color1) * 0.5,
            });
        if let Some(sound) = match event.power {
            ExplosionPower::None => None,
            ExplosionPower::Small => Some(&assets.explosion_small),
            ExplosionPower::Big => Some(&assets.explosion_big),
        } {
            sounds.send(Sound {
                sound: sound.clone(),
                position: Some(event.origin),
                ..default()
            });
        }
    }

    for (entity, state, mut light) in explosions.iter_mut() {
        let t = time.t_passed(state.start, state.e.time);
        if t >= 1. {
            commands.entity(entity).despawn_recursive();
            continue;
        }
        let radius = lerp(state.e.radius * 0.05, state.e.radius, t);
        let color = lerp(state.e.color0, state.e.color1, t);
        let alpha = 1. - t * t;
        let mut width = radius * 0.4;

        if radius + width >= state.e.radius {
            width = state.e.radius - radius
        }

        let radius = radius * scale;
        let width = width * scale;

        use bevy_lyon::*;
        commands.entity(entity).with_children(|parent| {
            parent
                .spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Circle {
                        radius: radius - width / 2.,
                        center: default(),
                    },
                    DrawMode::Stroke(StrokeMode::new(color.with_a(alpha), width)),
                    default(),
                ))
                .insert(TemporaryHack);
        });
        light.color.set_a(alpha * 0.3);
        light.radius = radius * 1.2;
    }

    for entity in hack.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn death_explosion(
    mut events: EventWriter<Explosion>, mut deaths: CmdReader<DeathEvent>,
    mut entities: Query<(&GlobalTransform, &DeathExplosion)>,
) {
    deaths.iter_entities(&mut entities, |_, (pos, explosion)| {
        let mut event = explosion.0;
        event.origin = pos.pos_2d();
        events.send(event)
    })
}

fn spawn_effect(
    mut events: EventWriter<Explosion>,
    entities: Query<(&GlobalTransform, &SpawnEffect), Added<SpawnEffect>>,
) {
    for (pos, effect) in entities.iter() {
        events.send(Explosion {
            origin: pos.pos_2d(),
            color0: Color::rgb(0.8, 1., 1.),
            color1: Color::rgb(0.8, 1., 1.),
            time: Duration::from_millis(400),
            radius: effect.radius,
            power: ExplosionPower::None,
        })
    }
}

#[derive(Component)]
struct FlashState {
    start: Duration,
    child: Entity,
}

fn spawn_flash(
    mut commands: Commands, entities: Query<(Entity, &Flash), Added<Flash>>, time: Res<GameTime>,
) {
    for (entity, flash) in entities.iter() {
        let mut hack = BadEntityHack::default();
        commands
            .entity(entity)
            .with_children(|parent| {
                use bevy_lyon::*;
                hack.set(
                    parent
                        .spawn_bundle(GeometryBuilder::build_as(
                            &shapes::Circle {
                                radius: flash.radius,
                                center: default(),
                            },
                            DrawMode::Fill(FillMode::color(flash.color0)),
                            default(),
                        ))
                        .insert(Depth::ImportantEffect)
                        .id(),
                );
            })
            .insert(FlashState {
                start: time.now(),
                child: hack.get(),
            });
    }
}

fn update_flash(
    mut commands: Commands, entities: Query<(Entity, &Flash, &FlashState)>,
    mut flashes: Query<&mut bevy_lyon::DrawMode>, time: Res<GameTime>,
) {
    for (entity, flash, state) in entities.iter() {
        let t = time.t_passed(state.start, flash.duration);
        if t >= 1. {
            commands.entity(state.child).despawn_recursive();
            commands
                .entity(entity)
                .remove::<Flash>()
                .remove::<FlashState>();
        } else {
            if let Ok(mut draw) = flashes.get_mut(state.child) {
                use bevy_lyon::*;
                *draw = DrawMode::Fill(FillMode::color(lerp_color(flash.color0, flash.color1, t)))
            }
        }
    }
}

#[derive(Component)]
struct Spark {
    velocity: Vec2,
    angular: f32,
    start: Duration,
    duration: Duration,
}

fn hit_sparks(
    mut commands: Commands, mut events: EventReader<HitSparks>,
    mut sparks: Query<(Entity, &Spark, &mut Transform)>, time: Res<GameTime>,
) {
    // new sparks
    for event in events.iter() {
        use bevy_lyon::*;
        use rand::*;

        let count = (event.damage as usize * 3).min(10);
        for _ in 0..count {
            let radius = thread_rng().gen_range(0.02..0.15);
            let offset = thread_rng().gen_range(0.5..0.7);
            let dir = Vec2::X.rotated(thread_rng().gen_range(0. ..TAU));

            let k_star = 0.3;
            commands
                .spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Polygon {
                        // star shape
                        points: vec![
                            vec2(radius, 0.),
                            vec2(radius * k_star, radius * k_star),
                            vec2(0., radius),
                            vec2(-radius * k_star, radius * k_star),
                            vec2(-radius, 0.),
                            vec2(-radius * k_star, -radius * k_star),
                            vec2(0., -radius),
                            vec2(radius * k_star, -radius * k_star),
                        ],
                        closed: true,
                    },
                    DrawMode::Fill(FillMode::color(Color::YELLOW)),
                    Transform::new_2d(event.origin + dir * offset),
                ))
                .insert(Depth::ImportantEffect)
                .insert(Spark {
                    velocity: dir * thread_rng().gen_range(1. ..3.),
                    angular: thread_rng().gen_range(-1. ..1.) * 2. * TAU,
                    start: time.now(),
                    duration: Duration::from_secs_f32(thread_rng().gen_range(0.6..1.2)),
                })
                .insert(GameplayObject);
        }
    }

    // update sparks
    for (entity, spark, mut transform) in sparks.iter_mut() {
        let t = time.t_passed(spark.start, spark.duration);
        if t >= 1. {
            commands.entity(entity).despawn_recursive();
        } else {
            transform.add_2d(spark.velocity * time.delta_seconds());
            let new_angle = transform.angle_2d() + spark.angular * time.delta_seconds();
            transform.set_angle_2d(new_angle);
            transform.scale = Vec3::new(1. - t, 1. - t, 1.);
        }
    }
}

fn hit_sparks_on_damage(
    mut sparks: EventWriter<HitSparks>, mut damage: CmdReader<ReceivedDamage>,
    mut entities: Query<&GlobalTransform, (Or<(With<Health>, With<Damage>)>, Without<DontSparkMe>)>,
    mut death: CmdReader<DeathEvent>,
) {
    damage.iter_entities(&mut entities, |event, _| {
        sparks.send(HitSparks {
            origin: event.point,
            damage: event.damage.value,
        })
    });
    // this is intended for projectiles which hit walls
    death.iter_entities(&mut entities, |_, pos| {
        sparks.send(HitSparks {
            origin: pos.pos_2d(),
            damage: 1.5,
        })
    });
}

#[derive(Component)]
struct RayState {
    effect: RayEffect,
    start: Duration,
}

fn ray(
    mut commands: Commands, new: Query<(Entity, &RayEffect), Added<RayEffect>>,
    mut rays: Query<(Entity, &RayState, &mut bevy_lyon::DrawMode, &Parent)>, time: Res<GameTime>,
) {
    use bevy_lyon::*;

    // new rays
    for (entity, effect) in new.iter() {
        commands.entity(entity).with_children(|parent| {
            parent
                .spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Line(Vec2::ZERO, Vec2::Y * effect.length),
                    DrawMode::Stroke(StrokeMode::new(effect.color, effect.width)),
                    default(),
                ))
                .insert(Depth::ImportantEffect)
                .insert(RayState {
                    effect: *effect,
                    start: time.now(),
                });
        });
    }

    // update rays
    for (entity, state, mut draw, parent) in rays.iter_mut() {
        let t = time.t_passed(state.start, state.effect.duration);
        if t >= 1. {
            let t = time.t_passed(state.start + state.effect.duration, state.effect.fade_time);
            if t >= 1. {
                if state.effect.destroy_parent {
                    commands.entity(parent.get()).despawn_recursive();
                } else {
                    commands.entity(entity).despawn_recursive();
                }
            } else {
                let t = 1. - t;
                *draw = DrawMode::Stroke(StrokeMode::new(
                    state.effect.color.with_a(t),
                    state.effect.width * t,
                ));
            }
        }
    }
}

fn flash_on_damage(
    mut commands: Commands, mut player: Query<(Entity, &FlashOnDamage)>,
    mut events: CmdReader<ReceivedDamage>,
) {
    let duration = Duration::from_millis(500);
    events.iter_entities(&mut player, |_, (entity, flash)| match *flash {
        FlashOnDamage::Radius(radius) => {
            commands.entity(entity).insert(Flash {
                radius,
                duration,
                color0: Color::RED,
                color1: Color::NONE,
            });
        }
    });
}

#[derive(Component)]
struct ChargingSpark {
    delta: Vec2,
    origin: Vec2,
    start: Duration,
    radius: f32,
}

#[derive(Component)]
struct ChargingState {
    start: Duration,
    inner: Entity,
    outer: Entity,
    count: usize,
}

fn charging_attack(
    mut commands: Commands, new: Query<(Entity, &ChargingAttack), Added<ChargingAttack>>,
    mut sparks: Query<(Entity, &mut Transform, &ChargingSpark)>,
    mut stat: Query<(
        Entity,
        &GlobalTransform,
        &mut ChargingState,
        &ChargingAttack,
    )>,
    mut hints: Query<(&mut Transform, &mut DrawMode), Without<ChargingSpark>>, time: Res<GameTime>,
    mut explode: EventWriter<Explosion>,
) {
    use bevy_lyon::*;

    for (entity, attack) in new.iter() {
        let mut inner = BadEntityHack::default();
        let mut outer = BadEntityHack::default();
        commands
            .entity(entity)
            .with_children(|parent| {
                inner.set(
                    parent
                        .spawn_bundle(GeometryBuilder::build_as(
                            &shapes::Circle {
                                radius: attack.radius,
                                center: Vec2::ZERO,
                            },
                            DrawMode::Fill(FillMode::color(Color::NONE)),
                            default(),
                        ))
                        .insert(Depth::ImportantEffect)
                        .id(),
                );
                outer.set(
                    parent
                        .spawn_bundle(GeometryBuilder::build_as(
                            &shapes::Circle {
                                radius: attack.radius,
                                center: Vec2::ZERO,
                            },
                            DrawMode::Stroke(StrokeMode::new(Color::NONE, 0.)),
                            default(),
                        ))
                        .insert(Depth::ImportantEffect)
                        .id(),
                );
            })
            .insert(ChargingState {
                start: time.now(),
                inner: inner.get(),
                outer: outer.get(),
                count: 0,
            });
    }

    for (entity, pos, mut state, attack) in stat.iter_mut() {
        let pos = pos.pos_2d();
        let t = time.t_passed(state.start, attack.duration);
        if t >= 1. {
            commands
                .entity(entity)
                .remove::<ChargingState>()
                .remove::<ChargingAttack>();
            commands.entity(state.inner).despawn_recursive();
            commands.entity(state.outer).despawn_recursive();

            explode.send(Explosion {
                origin: pos,
                color0: attack.color,
                color1: attack.color.with_a(0.),
                time: Duration::from_millis(300),
                radius: attack.radius * 0.5,
                power: ExplosionPower::None,
            })
        } else {
            if let Ok((mut transform, mut draw)) = hints.get_mut(state.inner) {
                transform.set_scale_2d(t);
                *draw = DrawMode::Fill(FillMode::color(attack.color.with_a(lerp(0.5, 1., t))));
            }
            if let Ok((mut transform, mut draw)) = hints.get_mut(state.outer) {
                transform.set_scale_2d(2. - t);
                *draw =
                    DrawMode::Stroke(StrokeMode::new(attack.color.with_a(lerp(0.5, 1., t)), 0.15));
            }

            let new_count =
                time.t_passed(state.start, Duration::from_secs_f32(lerp(0.15, 0.033, t))) as usize;
            for _ in state.count..new_count {
                let offset = (Vec2::Y * attack.radius * thread_rng().gen_range(0.5..2.))
                    .rotated(thread_rng().gen_range(0. ..TAU));
                use rand::*;
                commands
                    .spawn_bundle(GeometryBuilder::build_as(
                        &shapes::Circle {
                            radius: thread_rng().gen_range(0.05..0.15),
                            center: Vec2::ZERO,
                        },
                        DrawMode::Fill(FillMode::color(Color::NONE)),
                        Transform::new_2d(pos + offset),
                    ))
                    .insert(GameplayObject)
                    .insert(Depth::Effect)
                    .insert(ChargingSpark {
                        delta: offset,
                        origin: pos,
                        start: time.now(),
                        radius: attack.radius,
                    });
            }
            state.count = new_count;
        }
    }

    for (entity, mut transform, spark) in sparks.iter_mut() {
        let speed = spark.radius / 2. * time.passed(spark.start).as_secs_f32().powi(2);
        transform.add_2d(-spark.delta * speed);

        let delta = transform.pos_2d() - spark.origin;
        if delta.x * spark.delta.x <= 0. && delta.y * spark.delta.y <= 0. {
            commands.entity(entity).despawn_recursive()
        }
    }
}
