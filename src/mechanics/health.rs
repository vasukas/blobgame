use super::damage::Team;
use crate::common::*;

/// Entity will be despawned after death in First
#[derive(Component)]
pub struct Health {
    pub value: f32,
    pub max: f32,

    pub invincible: bool,
    pub armor: bool, // reduced damage from explosions
}

impl Health {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            max: value,
            invincible: false,
            armor: false,
        }
    }

    pub fn armor(mut self) -> Self {
        self.armor = true;
        self
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
    pub explosion: bool,
}

impl Damage {
    pub fn new(value: f32) -> Self {
        Self { value, ..default() }
    }

    /// Explodes projectiles
    pub fn powerful(mut self) -> Self {
        self.powerful = true;
        self
    }

    /// Reduced damage to some entities
    pub fn explosion(mut self) -> Self {
        self.explosion = true;
        self
    }
}

/// Entity event
pub struct DamageEvent {
    pub damage: Damage,
    pub team: Team,
}

/// Entity event
pub struct ReceivedDamage {
    pub damage: Damage,
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
            .add_system(die_after)
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
    mut death: CmdWriter<DeathEvent>, mut received: CmdWriter<ReceivedDamage>,
) {
    damage.iter_cmd_mut(&mut entities, |event, (entity, mut health, team)| {
        if *team != event.team && !health.invincible {
            health.value -=
                event.damage.value * if event.damage.explosion && health.armor { 0.5 } else { 1. };
            if health.value <= 0. {
                // TODO: statistics
                death.send((entity, DeathEvent))
            }
            received.send((
                entity,
                ReceivedDamage {
                    damage: event.damage,
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
