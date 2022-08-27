use crate::{
    common::*,
    mechanics::health::{DeathEvent, DieAfter, Health},
    present::simple_sprite::SimpleSprite,
};

/// If present, entity will drop that on death
#[derive(Component, Clone, Copy)]
pub enum Loot {
    Health { value: f32 },
}

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

fn drop_loot(
    mut death: CmdReader<DeathEvent>, mut commands: Commands,
    mut entities: Query<(&GlobalTransform, &Loot)>, assets: Res<MyAssets>,
) {
    death.iter_cmd_mut(&mut entities, |_, (pos, loot)| {
        let radius = 0.3;
        let lifetime = Duration::from_secs(8);

        commands
            .spawn_bundle(SpatialBundle::from_transform(Transform::new_2d(
                pos.pos_2d(),
            )))
            .insert(GameplayObject)
            .insert(RigidBody::Fixed)
            .insert(Collider::ball(radius))
            .insert(PhysicsType::Loot.rapier())
            .insert(Depth::Player)
            .insert(SimpleSprite {
                images: assets.player.clone(),
                frame_duration: Duration::from_millis(200),
                size: Vec2::splat(radius * 2.),
                ..default()
            })
            .with_children(|parent| {
                use bevy_lyon::*;
                parent.spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Circle {
                        radius: radius * 0.9,
                        center: Vec2::ZERO,
                    },
                    DrawMode::Fill(FillMode::color(match loot {
                        Loot::Health { .. } => Color::GREEN * 0.8,
                    })),
                    default(),
                ));
            })
            .insert(PickableLoot(*loot))
            .insert(DieAfter::new(lifetime));
    })
}

fn pick_loot(
    mut commands: Commands, phy: Res<RapierContext>,
    mut picker: Query<(&GlobalTransform, &mut Health, &mut LootPicker)>,
    loot: Query<&PickableLoot>,
) {
    for (pos, mut health, picker) in picker.iter_mut() {
        phy.intersections_with_shape(
            pos.pos_2d(),
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
                            }
                        }
                    }
                }
                false
            },
        );
    }
}
