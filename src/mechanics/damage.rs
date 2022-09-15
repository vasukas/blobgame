use super::{health::*, physics::CollectContacts};
use crate::{
    common::*,
    objects::stats::Stats,
    present::effect::{DontSparkMe, Explosion, ExplosionPower, RayEffect},
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
    pub retarget_on_wall: bool,
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
    let huge_distance = 1000.;
    let retarget_radius = 50.; // TODO: reduce?

    // TODO: refactor, this is fucking mess of broken logic

    for (source, pos, ray, mut damage, team) in rays.iter_mut() {
        let dir = Vec2::Y.rotated(pos.angle_2d());
        let filter = QueryFilter::new().groups(PhysicsType::Hitscan.into());

        let mut best_targets = vec![];
        phy.intersections_with_ray(
            pos.pos_2d(),
            dir,
            huge_distance,
            true,
            filter,
            |entity, intersect| {
                let add = match targets.get(entity) {
                    Ok((other_team, health, ..)) => {
                        !team.is_same(*other_team) && !health.invincible()
                    }
                    _ => true,
                };
                if add {
                    best_targets.push((entity, intersect.toi, intersect.point))
                }
                true
            },
        );
        best_targets.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap()); // by distance

        let mut max_distance = 0.;
        let mut prev_point = None;
        for (hit_entity, distance, point) in best_targets {
            let mut break_this = false;
            let mut hit_this = true;

            if let Ok((_, health, projectile, mut explode, ..)) = targets.get_mut(hit_entity) {
                if team.is_player() {
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
                                point,
                            },
                        ));
                    } else if projectile.is_some() {
                        stats.player.points += 1
                    }
                }
                if health.value < damage.value {
                    let new_damage =
                        damage.value - health.value - if health.max > 5. { 2. } else { 1. };
                    if new_damage < 0. {
                        break_this = true;
                        hit_this = false;
                    }
                    damage.value = new_damage
                }
                prev_point = Some(point);
            } else {
                if ray.retarget_on_wall {
                    if let Some(point) = prev_point {
                        let mut best = None;
                        phy.intersections_with_shape(
                            pos.pos_2d(),
                            pos.angle_2d(),
                            &Collider::ball(retarget_radius),
                            filter,
                            |entity| {
                                if let Ok((target_team, .., enemy, target_pos)) =
                                    targets.get(entity)
                                {
                                    if !target_team.is_same(*team) {
                                        match enemy {
                                            Some(EnemyType::Normal) => {
                                                let dir = target_pos.pos_2d() - pos.pos_2d();
                                                let (impact, distance) = phy
                                                    .cast_ray(
                                                        pos.pos_2d(),
                                                        dir,
                                                        1.,
                                                        false,
                                                        filter
                                                            .exclude_rigid_body(hit_entity)
                                                            .predicate(&|entity| match targets
                                                                .get(entity)
                                                            {
                                                                Ok((.., enemy, _)) => {
                                                                    enemy.is_some()
                                                                }
                                                                Err(_) => true,
                                                            }),
                                                    )
                                                    .unwrap_or((entity, dir.length_squared()));
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
                        println!("{:?}", best);
                        if let Some((.., Some(dir))) = best {
                            commands
                                .spawn_bundle(
                                    Transform::new_2d(point).with_angle_2d(dir.angle()).bundle(),
                                )
                                .insert(GameplayObject)
                                .insert(*damage)
                                .insert(ray.clone())
                                .insert(DieAfter::one_frame())
                                .insert(DontSparkMe)
                                .insert(*team);
                            hit_this = false;
                        }
                    }
                }
                if ray.ignore_obstacles {
                    hit_this = false;
                } else {
                    break_this = true
                }
            }

            if hit_this {
                max_distance = distance;
                if let Some(mut explosion) = ray.explosion_effect {
                    explosion.origin = point;
                    explode.send(explosion)
                }
                damage_cmd.send((
                    hit_entity,
                    DamageEvent {
                        source,
                        damage: *damage,
                        team: *team,
                        point,
                    },
                ));
            }
            if break_this {
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
