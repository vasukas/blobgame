use super::{health::*, physics::CollectContacts};
use crate::{
    common::*,
    objects::stats::Stats,
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
    pub ignore_obstacles: bool,
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
    mut rays: Query<(&GlobalTransform, &DamageRay, &mut Damage, &Team)>,
    targets: Query<(&Team, &Health)>, mut damage_cmd: CmdWriter<DamageEvent>,
    phy: Res<RapierContext>, mut commands: Commands, mut explode: EventWriter<Explosion>,
    mut stats: ResMut<Stats>,
) {
    let huge_distance = 1000.;

    for (pos, ray, mut damage, team) in rays.iter_mut() {
        let dir = Vec2::Y.rotated(pos.angle_2d());

        let mut best_targets = vec![];
        phy.intersections_with_ray(
            pos.pos_2d(),
            dir,
            huge_distance,
            true,
            QueryFilter::new().groups(PhysicsType::Hitscan.into()),
            |entity, intersect| {
                let same_team = targets
                    .get(entity)
                    .map(|other_team| *team == *other_team.0)
                    .unwrap_or(false);
                if !same_team {
                    best_targets.push((entity, intersect.toi, intersect.point))
                }
                true
            },
        );
        best_targets.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap()); // by distance

        let mut max_distance = 0.;
        for (entity, distance, point) in best_targets {
            max_distance = distance;
            if let Some(mut explosion) = ray.explosion_effect {
                explosion.origin = point;
                explode.send(explosion)
            }

            damage_cmd.send((
                entity,
                DamageEvent {
                    damage: *damage,
                    team: *team,
                    point,
                },
            ));

            if let Ok((_, health)) = targets.get(entity) {
                // hack to get points for shooting projectiles
                if *team == Team::Player {
                    if health.max < 1. {
                        stats.player.points += 1
                    }
                }
                if health.value < damage.value {
                    let new_damage =
                        damage.value - health.value - if health.max > 5. { 2. } else { 1. };
                    if new_damage < 0. {
                        break;
                    }
                    damage.value = new_damage
                }
            } else if !ray.ignore_obstacles {
                break;
            }
        }
        if let Some(mut effect) = ray.spawn_effect {
            effect.length = max_distance;
            effect.destroy_parent = true;
            commands
                .spawn_bundle(SpatialBundle::from_transform((*pos).into()))
                .insert(GameplayObject)
                .insert(effect);
        }
    }
}
