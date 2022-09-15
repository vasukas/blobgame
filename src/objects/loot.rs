use super::{player::Player, stats::Stats, weapon::CraftedWeapon};
use crate::{
    common::*,
    mechanics::{
        health::{DeathEvent, DieAfter, Health},
        movement::DropSpread,
    },
    present::sound::PlaySound,
};

#[derive(Clone, Copy)]
pub enum Loot {
    Health(f32),
    /// If None, random one will be selected
    Weapon(Option<CraftedWeapon>),
}

/// List of which loot entity can drop on death and what is the drop chance
#[derive(Component)]
pub struct DropsLoot(pub Vec<(Loot, f32)>);

#[derive(Component)]
pub struct PickableLoot(pub Loot);

#[derive(Component, Default)]
pub struct LootPicker {
    pub radius: f32,
}

//

pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, drop_loot)
            .add_system(pick_loot.exclusive_system());
    }
}

fn adjust_loot(
    (loot, chance): (Loot, f32), player: Option<&Health>, _stats: &Stats,
) -> (Loot, f32) {
    let k_health_chance = 1.5; // chance multiplier to get health if HP <= 50% by x1.5

    match loot {
        Loot::Health(_) => (
            loot,
            player
                .and_then(|hp| (hp.t() <= 0.5).then_some(chance * k_health_chance))
                .unwrap_or(chance),
        ),
        Loot::Weapon(weapon) => match weapon {
            Some(_) => (loot, chance),
            None => (
                Loot::Weapon(Some(CraftedWeapon::variants().random())),
                chance,
            ),
        },
    }
}

fn drop_loot(
    mut death: CmdReader<DeathEvent>, mut commands: Commands,
    mut entities: Query<(&GlobalTransform, &DropsLoot)>, player: Query<&Health, With<Player>>,
    stats: Res<Stats>,
) {
    death.iter_entities(&mut entities, |_, (pos, loot)| {
        for loot in &loot.0 {
            let (loot, chance) = adjust_loot(*loot, player.get_single().ok(), &stats);
            use rand::*;
            if !thread_rng().gen_bool(chance.clamp(0., 1.) as f64) {
                continue;
            }

            let (radius, color) = match loot {
                Loot::Health(..) => (0.3, Color::GREEN * 0.8),
                Loot::Weapon(..) => (0.4, Color::ORANGE_RED * 0.8),
            };
            let lifetime = Duration::from_secs(8);

            commands
                .spawn_bundle(SpatialBundle::from_transform(Transform::new_2d(
                    pos.pos_2d(),
                )))
                .insert(GameplayObject)
                .insert(RigidBody::KinematicPositionBased)
                .insert(Collider::ball(radius))
                .insert(PhysicsType::Loot.rapier())
                .insert(Depth::Player)
                .with_children(|parent| {
                    use bevy_lyon::*;
                    match loot {
                        Loot::Health(..) => {
                            parent.spawn_bundle(GeometryBuilder::build_as(
                                &shapes::Circle {
                                    radius: radius * 0.9,
                                    center: Vec2::ZERO,
                                },
                                DrawMode::Fill(FillMode::color(color)),
                                default(),
                            ));
                        }
                        Loot::Weapon(..) => {
                            parent.spawn_bundle(GeometryBuilder::build_as(
                                &shapes::Polygon {
                                    points: vec![
                                        vec2(radius, radius),
                                        vec2(-radius, radius),
                                        vec2(-radius, -radius),
                                        vec2(radius, -radius),
                                    ],
                                    closed: true,
                                },
                                DrawMode::Fill(FillMode::color(color)),
                                default(),
                            ));
                        }
                    };
                })
                .insert(PickableLoot(loot))
                .insert(DieAfter::new(lifetime))
                .insert(DropSpread::default());
        }
    })
}

fn pick_loot(
    mut commands: Commands, phy: Res<RapierContext>,
    mut picker: Query<(&GlobalTransform, &mut Health, &mut LootPicker)>,
    loot: Query<&PickableLoot>, mut sounds: EventWriter<PlaySound>, assets: Res<MyAssets>,
    mut stats: ResMut<Stats>,
) {
    for (pos, mut health, picker) in picker.iter_mut() {
        let pos = pos.pos_2d();
        phy.intersections_with_shape(
            pos,
            0.,
            &Collider::ball(picker.radius),
            QueryFilter::new().groups(PhysicsType::Loot.into()),
            |entity| {
                if let Ok(loot) = loot.get(entity) {
                    //
                    match loot.0 {
                        Loot::Health(value) => {
                            let new_health = (health.value + value).min(health.max);
                            if new_health > health.value {
                                health.value = new_health;

                                commands.entity(entity).despawn_recursive();
                                sounds.send(PlaySound::ui(assets.ui_pickup.clone()));
                            }
                        }

                        Loot::Weapon(weapon) => {
                            if let Some(weapon) = weapon {
                                *stats.weapon_mut() = Some((weapon, weapon.description().2))
                            } else {
                                log::warn!("None-weapon");
                            }

                            commands.entity(entity).despawn_recursive();
                            sounds.send(PlaySound::ui(assets.ui_pickup.clone()));
                        }
                    }
                }
                false
            },
        );
    }
}
