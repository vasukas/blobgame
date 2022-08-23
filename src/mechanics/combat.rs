use crate::common::*;

/// Event
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CombatEvent {
    Start,
    End,
    Died,
}

#[derive(Component)]
pub struct ArenaDoor;

//

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CombatEvent>()
            .add_system(arena_doors.exclusive_system());
    }
}

fn arena_doors(
    mut commands: Commands, mut doors: Query<(Entity, &mut Visibility), With<ArenaDoor>>,
    mut events: EventReader<CombatEvent>,
) {
    for event in events.iter() {
        match event {
            // TODO: add fade instead of simple visibility switch or some other effect
            CombatEvent::Start => {
                for (entity, mut vis) in doors.iter_mut() {
                    commands.entity(entity).insert(RigidBody::Fixed);
                    vis.is_visible = true;
                }
            }
            CombatEvent::End => {
                for (entity, mut vis) in doors.iter_mut() {
                    commands.entity(entity).remove::<RigidBody>();
                    vis.is_visible = false;
                }
            }
            CombatEvent::Died => (),
        }
    }
}
