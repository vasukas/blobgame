use super::damage::Team;
use crate::common::*;

/// Entity will be despawned after death in First
#[derive(Component)]
pub struct Health {
    pub value: f32,
    pub max: f32,

    pub invincible: bool,
    pub armor: bool, // reduced damage from explosions

    pub recent_damage: HashMap<Entity, Duration>,
}

impl Health {
    // damage from same source can't be dealt more frequently than that
    const DAMAGE_PERIOD: Duration = Duration::from_millis(500);

    pub fn new(value: f32) -> Self {
        Self {
            value,
            max: value,
            invincible: false,
            armor: false,
            recent_damage: default(),
        }
    }

    pub fn armor(mut self) -> Self {
        self.armor = true;
        self
    }

    /// How many health has relative to max, normally in 0-1 range
    pub fn t(&self) -> f32 {
        self.value / self.max
    }
}

#[derive(Component)]
pub struct DieAfter {
    time: Duration,
    after: Option<Duration>,
}

impl DieAfter {
    pub fn new(time: Duration) -> Self {
        Self { time, after: None }
    }

    pub fn one_frame() -> Self {
        Self::new(default())
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct Damage {
    pub value: f32,
    pub powerful: bool,
    /// Reduced damage to some entities
    pub explosion: bool,
}

impl Damage {
    pub fn new(value: f32) -> Self {
        Self { value, ..default() }
    }

    /// Explodes projectiles
    pub fn powerful(mut self, is: bool) -> Self {
        self.powerful = is;
        self
    }
}

/// Entity event
pub struct DamageEvent {
    pub source: Entity,
    pub damage: Damage,
    pub team: Team,
    pub point: Vec2,
}

/// Entity event
pub struct ReceivedDamage {
    pub damage: Damage,
    pub point: Vec2,
}

/// Entity event
#[derive(Default)]
pub struct DeathEvent;

//

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<(Entity, DeathEvent)>()
            .add_event::<(Entity, DamageEvent)>()
            .add_event::<(Entity, ReceivedDamage)>()
            .add_system_to_stage(CoreStage::Last, die_after)
            .add_system(damage)
            .add_system_to_stage(CoreStage::First, despawn_dead.exclusive_system());
    }
}

fn die_after(
    time: Res<GameTime>, mut entities: Query<(Entity, &mut DieAfter)>,
    mut death: CmdWriter<DeathEvent>,
) {
    for (entity, mut after) in entities.iter_mut() {
        match after.after {
            Some(after) => {
                // TODO: rework this to use damage events?
                if time.reached(after) {
                    death.send((entity, DeathEvent))
                }
            }
            None => after.after = Some(time.now() + after.time),
        }
    }
}

fn damage(
    mut damage: CmdReader<DamageEvent>, mut entities: Query<(Entity, &mut Health, &Team)>,
    mut death: CmdWriter<DeathEvent>, mut received: CmdWriter<ReceivedDamage>, time: Res<GameTime>,
) {
    damage.iter_cmd_mut(&mut entities, |event, (entity, mut health, team)| {
        if !team.is_same(event.team) && !health.invincible {
            health
                .recent_damage
                .retain(|_, at| time.passed(*at) < Health::DAMAGE_PERIOD);
            if health.recent_damage.contains_key(&event.source) {
                return;
            }
            health.recent_damage.insert(event.source, time.now());

            health.value -=
                event.damage.value * if event.damage.explosion && health.armor { 0.5 } else { 1. };
            if health.value <= 0. {
                death.send((entity, DeathEvent))
            }
            received.send((
                entity,
                ReceivedDamage {
                    damage: event.damage,
                    point: event.point,
                },
            ));
        }
    })
}

fn despawn_dead(mut commands: Commands, mut deaths: EventReader<(Entity, DeathEvent)>) {
    for (entity, _) in deaths.iter() {
        commands.entity(*entity).despawn_recursive()
    }
}
