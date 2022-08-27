use crate::common::*;

pub mod loot;
pub mod player;
pub mod spawn;
pub mod stats;
pub mod weapon;

pub struct ObjectsPlugin;

impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(spawn::SpawnPlugin)
            .add_plugin(player::PlayerPlugin)
            .add_plugin(weapon::WeaponPlugin)
            .add_plugin(stats::StatsPlugin)
            .add_plugin(loot::LootPlugin);
    }
}
