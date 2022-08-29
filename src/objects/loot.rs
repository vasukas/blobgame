use super::stats::Stats;
use crate::{
    common::*,
    control::input::InputAction,
    mechanics::{
        health::{DeathEvent, DieAfter, Health},
        movement::DropSpread,
    },
    present::sound::Sound,
};
use enum_map::Enum;

#[derive(Clone, Copy)]
pub enum Loot {
    Health { value: f32 },
    CraftPart(CraftPart),
}

/// If present, entity will drop that on death
#[derive(Component)]
pub struct DropsLoot(pub Vec<Loot>);

#[derive(Component)]
pub struct PickableLoot(pub Loot);

#[derive(Component, Default)]
pub struct LootPicker {
    pub radius: f32,
}

#[derive(Clone, Copy, Enum, Debug)]
pub enum CraftPart {
    Generator,
    Emitter,
    Laser,
    Magnet,
}

impl CraftPart {
    pub fn random() -> Self {
        [Self::Generator, Self::Emitter, Self::Laser, Self::Magnet]
            .into_iter()
            .random()
    }
    pub fn description(&self) -> (InputAction, &'static str, usize) {
        match self {
            CraftPart::Generator => (InputAction::CraftSelect1, "Generator", 1),
            CraftPart::Emitter => (InputAction::CraftSelect2, "Emitter", 1),
            CraftPart::Laser => (InputAction::CraftSelect3, "Laser", 2),
            CraftPart::Magnet => (InputAction::CraftSelect4, "Magnet", 2),
        }
    }
}

//

pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, drop_loot)
            .add_system(pick_loot.exclusive_system());
    }
}

fn drop_loot(
    mut death: CmdReader<DeathEvent>, mut commands: Commands,
    mut entities: Query<(&GlobalTransform, &DropsLoot)>,
) {
    death.iter_cmd_mut(&mut entities, |_, (pos, loot)| {
        for loot in &loot.0 {
            let (radius, color) = match loot {
                Loot::Health { .. } => (0.3, Color::GREEN * 0.8),
                Loot::CraftPart(_) => (0.4, Color::ORANGE_RED * 0.8),
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
                        Loot::Health { .. } => {
                            parent.spawn_bundle(GeometryBuilder::build_as(
                                &shapes::Circle {
                                    radius: radius * 0.9,
                                    center: Vec2::ZERO,
                                },
                                DrawMode::Fill(FillMode::color(color)),
                                default(),
                            ));
                        }
                        Loot::CraftPart(_) => {
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
                .insert(PickableLoot(*loot))
                .insert(DieAfter::new(lifetime))
                .insert(DropSpread::default());
        }
    })
}

fn pick_loot(
    mut commands: Commands, phy: Res<RapierContext>,
    mut picker: Query<(&GlobalTransform, &mut Health, &mut LootPicker)>,
    loot: Query<&PickableLoot>, mut sounds: EventWriter<Sound>, assets: Res<MyAssets>,
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
                        Loot::Health { value } => {
                            let new_health = (health.value + value).min(health.max);
                            if new_health > health.value {
                                health.value = new_health;

                                commands.entity(entity).despawn_recursive();
                                sounds.send(Sound {
                                    sound: assets.ui_pickup.clone(),
                                    position: Some(pos),
                                    non_randomized: true,
                                    ..default()
                                });
                            }
                        }

                        Loot::CraftPart(part) => {
                            stats.player.craft_parts[part] += 1;

                            commands.entity(entity).despawn_recursive();
                            sounds.send(Sound {
                                sound: assets.ui_pickup.clone(),
                                position: Some(pos),
                                non_randomized: true,
                                ..default()
                            });
                        }
                    }
                }
                false
            },
        );
    }
}
