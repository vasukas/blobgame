use super::{loot::CraftPart, spawn::WaveEvent, weapon::CraftedWeapon};
use crate::{common::*, mechanics::health::DeathEvent};
use enum_map::EnumMap;

#[derive(Default)]
pub struct Stats {
    pub wave: usize,
    pub time: Duration,
    pub restarts: usize,
    pub ubercharge: f32,

    pub player: PersistentPlayer,
    pub player_weapon_slot: usize, // which weapon slot used
    last_wave: PersistentPlayer,
}

impl Stats {
    pub fn weapon_mut(&mut self) -> &mut Option<(CraftedWeapon, f32)> {
        &mut self.player.weapons[self.player_weapon_slot]
    }
}

/// Stuff restored after respawn
#[derive(Clone)]
pub struct PersistentPlayer {
    pub points: usize,
    pub craft_parts: EnumMap<CraftPart, usize>, // count
    pub weapons: [Option<(CraftedWeapon, f32)>; 2], // usage left
}

impl Default for PersistentPlayer {
    fn default() -> Self {
        Self {
            points: Default::default(),
            craft_parts: enum_map::enum_map! {
                CraftPart::Generator => 2,
                CraftPart::Emitter => 2,
                CraftPart::Laser => 0,
                CraftPart::Magnet => 2,
            },
            weapons: [
                Some((CraftedWeapon::Railgun, 3.)),
                Some((CraftedWeapon::Plasma, 3.)),
            ],
        }
    }
}

#[derive(Component, Default)]
pub struct DeathPoints {
    pub value: usize,
    pub charge: f32,
}

//

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stats>()
            .add_system_to_stage(CoreStage::PostUpdate, update_stats);
    }
}

fn update_stats(
    mut stats: ResMut<Stats>, mut events: EventReader<WaveEvent>, time: Res<GameTime>,
    mut wave_now: Local<bool>, mut deaths: CmdReader<DeathEvent>, mut diers: Query<&DeathPoints>,
) {
    for ev in events.iter() {
        match ev {
            WaveEvent::Started => {
                stats.last_wave = stats.player.clone();
                *wave_now = true;
            }
            WaveEvent::Ended => {
                *wave_now = false;
            }
            WaveEvent::Restart => {
                stats.player = stats.last_wave.clone();
                stats.restarts += 1;
                stats.ubercharge = 0.;
            }
        }
    }
    if *wave_now {
        stats.time += time.delta()
    }

    deaths.iter_cmd_mut(&mut diers, |_, points| {
        stats.player.points += points.value;
        stats.ubercharge += points.charge;
    });
}
