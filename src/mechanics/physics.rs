use crate::common::*;
use bevy::utils::HashSet;

//

#[derive(Clone, Copy)]
pub enum PhysicsType {
    Obstacle,
    Player,
    Script,
    PlunkDown,
}

impl PhysicsType {
    pub fn rapier(self) -> CollisionGroups {
        let obstacles = 1;
        let player = 2;
        let script = 4;

        let (memberships, filters) = match self {
            PhysicsType::Obstacle => (obstacles, player),
            PhysicsType::Player => (player, obstacles | script),
            PhysicsType::Script => (script, player),
            PhysicsType::PlunkDown => (255, obstacles | script),
        };
        CollisionGroups {
            memberships,
            filters,
        }
    }
}

impl Into<InteractionGroups> for PhysicsType {
    fn into(self) -> InteractionGroups {
        self.rapier().into()
    }
}

//

/// Keep list of all current contacts
#[derive(Component, Default)]
pub struct CollectContacts {
    pub current: HashSet<Entity>,
}

//

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            collect_contacts_enable.exclusive_system().at_start(),
        )
        .add_system(collect_contacts);
    }
}

fn collect_contacts_enable(
    mut commands: Commands,
    entities: Query<(Entity, Option<&ActiveEvents>), Added<CollectContacts>>,
) {
    for (entity, events) in entities.iter() {
        if events.is_none() {
            commands
                .entity(entity)
                .insert(ActiveEvents::COLLISION_EVENTS);
        }
    }
}

fn collect_contacts(
    mut entities: Query<&mut CollectContacts>, mut events: EventReader<CollisionEvent>,
) {
    for event in events.iter() {
        let (new, a, b) = match event {
            CollisionEvent::Started(a, b, _) => (true, a, b),
            CollisionEvent::Stopped(a, b, _) => (false, a, b),
        };
        for (this, other) in [(a, b), (b, a)] {
            if let Ok(mut contacts) = entities.get_mut(*this) {
                if new {
                    contacts.current.insert(*other)
                } else {
                    contacts.current.remove(other)
                };
            }
        }
    }
}
