use crate::common::*;

/// Entity will be despawned after death in First
#[derive(Component)]
pub struct Health {
    pub value: f32,
    pub max: f32,
    pub invincible: bool,
}

impl Health {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            max: value,
            invincible: false,
        }
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

/// Entity event
pub struct DeathEvent;

//

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<(Entity, DeathEvent)>()
            .add_system(die_after)
            .add_system(death_event)
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

fn death_event(
    mut death: CmdWriter<DeathEvent>, health: Query<(Entity, &Health), Changed<Health>>,
) {
    for (entity, health) in health.iter() {
        if health.value <= 0. {
            death.send((entity, DeathEvent))
        }
    }
}

fn despawn_dead(mut commands: Commands, mut deaths: EventReader<(Entity, DeathEvent)>) {
    for (entity, _) in deaths.iter() {
        commands.entity(*entity).despawn_recursive()
    }
}
