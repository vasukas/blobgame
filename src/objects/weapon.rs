use crate::{
    common::*,
    mechanics::{damage::*, health::Damage},
};

/// Command event
#[derive(Clone, Copy, Default)]
pub enum Weapon {
    #[default]
    None,
    Turret,
    PlayerGun,
}

#[derive(SystemLabel)]
pub struct WeaponSystemLabel;

//

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<(Entity, Weapon)>()
            .add_system(weapon.label(WeaponSystemLabel));
    }
}

fn weapon(
    mut commands: Commands, mut weapon: CmdReader<Weapon>,
    mut source: Query<(&GlobalTransform, &Team)>,
) {
    use bevy_lyon::*;
    weapon.iter_cmd_mut(&mut source, |weapon, (transform, team)| match *weapon {
        Weapon::None => log::warn!("Shooting Weapon::None"),
        Weapon::Turret => {
            println!("WTF");

            let radius = 0.25;
            commands
                .spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Polygon {
                        points: vec![
                            vec2(radius * 3., 0.),
                            vec2(radius, 0.).rotated(160f32.to_radians()),
                            vec2(radius, 0.).rotated(-160f32.to_radians()),
                        ],
                        closed: true,
                    },
                    DrawMode::Fill(FillMode::color(Color::ORANGE_RED)),
                    forward(transform, 1.5),
                ))
                .insert(Depth::Projectile)
                .insert(GameplayObject)
                .insert(DamageCircle {
                    damage: Damage::new(1.),
                    radius,
                    team: *team,
                })
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(radius))
                .insert(PhysicsType::Projectile.rapier())
                .insert(Velocity::linear((transform.forward() * 10.).truncate()));
        }
        Weapon::PlayerGun => {
            // TODO: implement
        }
    });
}

fn forward(pos: &GlobalTransform, distance: f32) -> Transform {
    let mut pos: Transform = (*pos).into();
    pos.translation += pos.forward() * distance;
    pos
}
