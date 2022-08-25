use super::{health::*, physics::CollectContacts};
use crate::common::*;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    Player,
    Enemy,
}

/// Requires Damage
#[derive(Component)]
pub struct DamageOnContact;

#[derive(Component)]
pub struct DieOnContact;

//

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(damage_on_contact).add_system(die_on_contact);
    }
}

fn damage_on_contact(
    entities: Query<(&CollectContacts, &Damage, &Team), With<DamageOnContact>>,
    targets: Query<(), With<Health>>, mut damage_cmd: CmdWriter<DamageEvent>,
) {
    for (contacts, damage, team) in entities.iter() {
        for entity in contacts.current.iter().copied() {
            if targets.contains(entity) {
                damage_cmd.send((
                    entity,
                    DamageEvent {
                        damage: *damage,
                        team: *team,
                    },
                ))
            }
        }
    }
}

fn die_on_contact(
    entities: Query<(Entity, &CollectContacts), With<DieOnContact>>,
    mut death: CmdWriter<DeathEvent>,
) {
    for (entity, contacts) in entities.iter() {
        if !contacts.current.is_empty() {
            death.send((entity, default()))
        }
    }
}
