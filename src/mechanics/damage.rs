use super::{health::*, physics::CollectContacts};
use crate::{
    common::*,
    objects::stats::Stats,
    present::effect::{Explosion, ExplosionPower, RayEffect},
};

#[derive(Component, Clone, Copy)]
pub enum Team {
    Player,
    Enemy,
    YEEEEEEE,
}

#[derive(Component, Clone, Copy)]
pub enum EnemyType {
    Normal,
}

impl Team {
    pub fn is_same(&self, rhs: Team) -> bool {
        match (self, rhs) {
            (Team::Player, Team::Player) | (Team::Enemy, Team::Enemy) => true,
            (Team::YEEEEEEE, _) | (_, Team::YEEEEEEE) => false,
            _ => false,
        }
    }
    pub fn is_player(&self) -> bool {
        match self {
            Team::Player => true,
            _ => false,
        }
    }
}

/// Requires Damage
#[derive(Component)]
pub struct DamageOnContact;

#[derive(Component)]
pub struct DieOnContact;

/// Requires Damage
#[derive(Component, Default, Clone)]
pub struct DamageRay {
    pub spawn_effect: Option<RayEffect>, // length will be set on hit
    pub explosion_effect: Option<Explosion>, // show explosion where it hits
    pub ignore_obstacles: bool,
    pub bounce_before_wall: Option<f32>, // radius
    pub max_length: Option<f32>,         // or unlimited
}

#[derive(Component, Clone, Copy)]
pub struct ExplodeOnDeath {
    pub damage: f32,
    pub radius: f32,
    pub effect: Explosion,
    pub activated: bool,
}

#[derive(Component)]
pub struct SmallProjectile;

#[derive(Component)]
pub struct BigProjectile;

#[derive(Component)]
pub struct BonkToTeam(pub Team);

//

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(damage_on_contact)
            .add_system(die_on_contact)
            .add_system_to_stage(CoreStage::PostUpdate, damage_ray)
            .add_system_to_stage(CoreStage::PostUpdate, explode_on_death.after(damage_ray))
            .add_system_to_stage(CoreStage::Last, bonk_to_same_team);
    }
}

fn damage_on_contact(
    entities: Query<
        (Entity, &CollectContacts, &GlobalTransform, &Damage, &Team),
        With<DamageOnContact>,
    >,
    targets: Query<&GlobalTransform, With<Health>>, mut damage_cmd: CmdWriter<DamageEvent>,
    phy: Res<RapierContext>,
) {
    for (source, contacts, origin, damage, team) in entities.iter() {
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
                        source,
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
    entities: Query<(Entity, &CollectContacts, Option<&BigProjectile>), With<DieOnContact>>,
    mut death: CmdWriter<DeathEvent>, projectiles: Query<(), With<SmallProjectile>>,
    invincible: Query<&Health>,
) {
    for (entity, contacts, big) in entities.iter() {
        if match big.is_some() {
            true => {
                contacts.current.iter().any(|e| !projectiles.contains(*e))
                    && contacts.current.iter().any(|e| {
                        !invincible
                            .get(*e)
                            .map(|hp| hp.invincible())
                            .unwrap_or(false)
                    })
            }
            false => !contacts.current.is_empty(),
        } {
            death.send((entity, default()))
        }
    }
}

fn damage_ray(
    mut rays: Query<(Entity, &GlobalTransform, &DamageRay, &mut Damage, &Team)>,
    mut targets: Query<(
        &Team,
        &Health,
        Option<&SmallProjectile>,
        Option<&mut ExplodeOnDeath>,
        Option<&EnemyType>,
        &GlobalTransform,
    )>,
    mut damage_cmd: CmdWriter<DamageEvent>, phy: Res<RapierContext>, mut commands: Commands,
    mut explode: EventWriter<Explosion>, mut stats: ResMut<Stats>,
) {
    for (source, pos, ray, mut damage, team) in rays.iter_mut() {
        let mut filter = QueryFilter::new().groups(PhysicsType::Hitscan.into());
        let mut origin = pos.pos_2d();
        let mut dir = Vec2::Y.rotated(pos.angle_2d());

        let mut ray_targets = vec![];
        let mut main_raycast = true;

        while main_raycast {
            main_raycast = false;
            let original_origin = origin;
            ray_targets.clear();

            // find everything intersecting with ray
            phy.intersections_with_ray(
                origin,
                dir,
                ray.max_length.unwrap_or(f32::MAX),
                true,
                filter,
                |entity, intersect| {
                    if match targets.get(entity) {
                        Ok((other_team, health, ..)) => {
                            // different team and not invincible
                            !team.is_same(*other_team) && !health.invincible()
                        }
                        _ => true,
                    } {
                        ray_targets.push((entity, intersect.toi, intersect.point))
                    }
                    true
                },
            );
            ray_targets.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap()); // by distance

            // iterate over objects hit
            enum HitResult {
                Continue,
                End,
                EndPrevious,
            }
            let mut prev_hit = None;

            for (hit_entity, _, hit_point) in ray_targets.iter().copied() {
                let mut result = HitResult::Continue;

                // hit something destructible
                if let Ok((_, health, projectile, mut explode, enemy_type, ..)) =
                    targets.get_mut(hit_entity)
                {
                    // ray is produced by player
                    if team.is_player() {
                        // target can explode (i.e. big projectile)
                        if let Some(explode) = explode.as_mut().filter(|_| damage.powerful) {
                            stats.player.points += 50;
                            explode.activated = true;

                            // force death
                            damage_cmd.send((
                                hit_entity,
                                DamageEvent {
                                    source,
                                    damage: Damage::new(10000.),
                                    team: Team::YEEEEEEE,
                                    point: hit_point,
                                },
                            ));
                        }
                        // target is small projectile - destroyed
                        else if projectile.is_some() {
                            stats.player.points += 1
                        }
                    }

                    // check if ray will pass through the target
                    if health.value < damage.value {
                        let new_damage = damage.value
                            - health.value
                            - match enemy_type {
                                Some(EnemyType::Normal) => 1.,
                                None => 0.,
                            };
                        if new_damage > 0. {
                            damage.value = new_damage
                        } else {
                            result = HitResult::End;
                        }
                    }
                    // it won't
                    else {
                        result = HitResult::End;
                    }
                }
                // hit obstacle
                else {
                    // hit the wall
                    if !ray.ignore_obstacles {
                        result = HitResult::End
                    }

                    // bounce off to another target if possible, which requires any target hit first
                    if let Some((radius, (hit_point, hit_entity))) =
                        ray.bounce_before_wall.zip(prev_hit)
                    {
                        // find best (closest) target
                        // TODO: or maybe select targets based on angle?
                        let mut best = None;
                        phy.intersections_with_shape(
                            hit_point,
                            0.,
                            &Collider::ball(radius),
                            filter,
                            |entity| {
                                if let Ok((target_team, .., enemy, target_pos)) =
                                    targets.get(entity)
                                {
                                    if !target_team.is_same(*team) {
                                        match enemy {
                                            Some(EnemyType::Normal) => {
                                                // check if have LoS to target
                                                let dir = target_pos.pos_2d() - hit_point;
                                                let distance = dir.length();
                                                let dir = if distance > 0. {
                                                    dir / distance
                                                } else {
                                                    dir
                                                };

                                                let (impact, distance) = phy
                                                    .cast_ray(
                                                        hit_point,
                                                        dir,
                                                        radius,
                                                        true,
                                                        filter
                                                            // ignore entity from which ray starts
                                                            .exclude_rigid_body(hit_entity)
                                                            // ignore small projectiles
                                                            .predicate(&|entity| match targets
                                                                .get(entity)
                                                            {
                                                                Ok((.., projectile, __, _)) => {
                                                                    projectile.is_none()
                                                                }
                                                                Err(_) => true,
                                                            }),
                                                    )
                                                    .unwrap_or((entity, distance));

                                                // have LoS to target
                                                if impact == entity {
                                                    let impact = (
                                                        distance,
                                                        entity,
                                                        targets.contains(entity).then_some(dir),
                                                    );
                                                    let (best_distance, ..) =
                                                        best.get_or_insert(impact);
                                                    if distance < *best_distance {
                                                        best = Some(impact);
                                                    }
                                                }
                                            }
                                            None => (),
                                        }
                                    }
                                }
                                true
                            },
                        );
                        // found target
                        if let Some((.., Some(best_dir))) = best {
                            // ignore entity from which ray starts
                            filter = filter.exclude_rigid_body(hit_entity);

                            origin = hit_point;
                            dir = best_dir;
                            main_raycast = true;
                            result = HitResult::EndPrevious;
                        }
                    }
                }

                // generate effect and loop control
                let (break_this, was_hit) = match result {
                    HitResult::Continue => (false, true),
                    HitResult::End => (true, true),
                    HitResult::EndPrevious => (true, false),
                };
                if was_hit {
                    if let Some(mut explosion) = ray.explosion_effect {
                        explosion.origin = hit_point;
                        explode.send(explosion)
                    }
                    damage_cmd.send((
                        hit_entity,
                        DamageEvent {
                            source,
                            damage: *damage,
                            team: *team,
                            point: hit_point,
                        },
                    ));
                    prev_hit = Some((hit_point, hit_entity));
                }
                if break_this {
                    break;
                }
            }

            // visual effect
            if let Some((mut effect, (hit_point, _))) = ray.spawn_effect.zip(prev_hit) {
                let dir = hit_point - original_origin;
                effect.length = dir.length();
                effect.destroy_parent = true;
                commands
                    .spawn_bundle(
                        Transform::new_2d(original_origin)
                            .with_angle_2d(dir.angle())
                            .bundle(),
                    )
                    .insert(GameplayObject)
                    .insert(effect);
            }
        }
    }
}

fn explode_on_death(
    mut entities: Query<(Entity, &GlobalTransform, &ExplodeOnDeath)>,
    mut death: CmdReader<DeathEvent>, mut explode: EventWriter<Explosion>, phy: Res<RapierContext>,
    mut damage: CmdWriter<DamageEvent>, targets: Query<&GlobalTransform>,
) {
    death.iter_entities(&mut entities, |_, (source, pos, e)| {
        let pos = pos.pos_2d();

        let mut e = *e;
        e.effect.origin = pos;
        if e.activated {
            e.damage *= 2.;
            e.radius *= 1.5;

            e.effect.radius *= 1.5;
            e.effect.color0 = Color::WHITE;
            e.effect.color1 = Color::ORANGE_RED;
            e.effect.power = ExplosionPower::Big;

            e.effect.time = Duration::from_millis(250);
            explode.send(e.effect);
            e.effect.time = Duration::from_millis(1000);
        }

        phy.intersections_with_shape(
            pos,
            0.,
            &Collider::ball(e.radius),
            QueryFilter::new(),
            |entity| {
                damage.send((
                    entity,
                    DamageEvent {
                        source,
                        damage: Damage {
                            explosion: true,
                            ..Damage::new(e.damage)
                        },
                        team: Team::YEEEEEEE,
                        point: targets.get(entity).map(|v| v.pos_2d()).unwrap_or_default(),
                    },
                ));
                true
            },
        );
        explode.send(e.effect);
    });
}

fn bonk_to_same_team(
    bonker: Query<(&CollectContacts, &BonkToTeam)>,
    mut projectiles: Query<&mut Team, With<BigProjectile>>,
) {
    for (contacts, bonk) in bonker.iter() {
        for entity in &contacts.current {
            if let Ok(mut team) = projectiles.get_mut(*entity) {
                *team = bonk.0
            }
        }
    }
}
