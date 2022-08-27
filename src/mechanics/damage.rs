use super::{health::*, physics::CollectContacts};
use crate::{
    common::*,
    present::effect::{Explosion, RayEffect},
};

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

/// Requires Damage
#[derive(Component, Default)]
pub struct DamageRay {
    pub spawn_effect: Option<RayEffect>, // length will be set on hit
    pub explosion_effect: Option<Explosion>, // show explosion where it hits

                                         // TODO: other parameters
}

//

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(damage_on_contact)
            .add_system(die_on_contact)
            .add_system_to_stage(CoreStage::PostUpdate, damage_ray);
    }
}

fn damage_on_contact(
    entities: Query<(&CollectContacts, &GlobalTransform, &Damage, &Team), With<DamageOnContact>>,
    targets: Query<&GlobalTransform, With<Health>>, mut damage_cmd: CmdWriter<DamageEvent>,
    phy: Res<RapierContext>,
) {
    for (contacts, origin, damage, team) in entities.iter() {
        for entity in contacts.current.iter().copied() {
            if let Ok(pos) = targets.get(entity) {
                let origin = origin.pos_2d();
                let dir = pos.pos_2d() - origin;

                let toi = phy
                    .cast_ray(
                        origin,
                        dir,
                        1.,
                        true,
                        QueryFilter::new().predicate(&|f_entity| f_entity == entity),
                    )
                    .map(|v| v.1)
                    .unwrap_or(1.);

                damage_cmd.send((
                    entity,
                    DamageEvent {
                        damage: *damage,
                        team: *team,
                        point: origin + toi * dir,
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

fn damage_ray(
    rays: Query<(&GlobalTransform, &DamageRay, &Damage, &Team)>,
    targets: Query<&Team, With<Health>>, mut damage_cmd: CmdWriter<DamageEvent>,
    phy: Res<RapierContext>, mut commands: Commands, mut explode: EventWriter<Explosion>,
) {
    let huge_distance = 1000.;

    for (pos, ray, damage, team) in rays.iter() {
        let mut best_distance = huge_distance;
        let mut best_target = None;

        let dir = Vec2::Y.rotated(pos.angle_2d());

        phy.intersections_with_ray(
            pos.pos_2d(),
            dir,
            huge_distance,
            true,
            QueryFilter::new().groups(PhysicsType::Hitscan.into()),
            |entity, intersect| {
                let same_team = targets
                    .get(entity)
                    .map(|other_team| *team == *other_team)
                    .unwrap_or(false);
                if intersect.toi < best_distance && !same_team {
                    best_distance = intersect.toi;
                    best_target = Some((entity, intersect.point));
                }
                true
            },
        );
        if let Some((entity, point)) = best_target {
            damage_cmd.send((
                entity,
                DamageEvent {
                    damage: *damage,
                    team: *team,
                    point,
                },
            ));
            if let Some(mut effect) = ray.spawn_effect {
                effect.length = best_distance;
                effect.destroy_parent = true;
                commands
                    .spawn_bundle(SpatialBundle::from_transform((*pos).into()))
                    .insert(GameplayObject)
                    .insert(effect);
            }
            if let Some(mut explosion) = ray.explosion_effect {
                explosion.origin = point;
                explode.send(explosion)
            }
        } else {
            log::warn!("ray didn't hit anything!");
        }
    }
}
