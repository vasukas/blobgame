use crate::{common::*, present::sound::Sound};

/// Command event
#[derive(Clone, Copy, Default)]
pub enum Weapon {
    #[default]
    None,
    TurretTest,
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
    mut source: Query<(Entity, &GlobalTransform)>, mut sounds: EventWriter<Sound>,
    server: Res<AssetServer>,
) {
    weapon.iter_cmd_mut(&mut source, |weapon, (entity, transform)| match *weapon {
        Weapon::None => log::warn!("Shooting Weapon::None"),
        Weapon::TurretTest => {
            sounds.send(Sound {
                sound: server.load("sounds/explosion_bot_1.ogg"),
                position: Some(transform.pos_2d()),
            });
            // TODO: implement actual weapons
        }
    });
}
