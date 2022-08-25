use super::{light::Light, sound::Sound};
use crate::{common::*, mechanics::health::DeathEvent};

// Event
#[derive(Clone, Copy)]
pub struct Explosion {
    pub origin: Vec2,
    pub color0: Color,
    pub color1: Color,
    pub time: Duration,
    pub radius: f32,
    pub power: ExplosionPower,
}

#[derive(Clone, Copy)]
pub enum ExplosionPower {
    None,
    Small,
}

impl Explosion {
    pub fn death(self) -> DeathExplosion {
        DeathExplosion(self)
    }
}

#[derive(Component)]
pub struct DeathExplosion(pub Explosion);

#[derive(Component)]
pub struct SpawnEffect {
    pub radius: f32,
}

//

pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Explosion>()
            .add_system(explosion.exclusive_system())
            .add_system_to_stage(CoreStage::PostUpdate, death_explosion)
            .add_system_to_stage(CoreStage::Last, spawn_effect);
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
    for event in events.iter() {
        commands
            .spawn_bundle(SpatialBundle::from_transform(Transform::new_2d(
                event.origin,
            )))
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
        } {
            sounds.send(Sound {
                sound: sound.clone(),
                position: Some(event.origin),
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
        let width = radius * 0.4;

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
    deaths.iter_cmd_mut(&mut entities, |_, (pos, explosion)| {
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
