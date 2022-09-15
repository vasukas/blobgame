use super::{spawn::WaveEvent, weapon::CraftedWeapon};
use crate::{common::*, mechanics::health::DeathEvent};

#[derive(Default)]
pub struct Stats {
    pub wave: usize,
    pub time: Duration,
    pub restarts: usize,

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
    pub weapons: [Option<(CraftedWeapon, f32)>; 2], // usage left
}

impl Default for PersistentPlayer {
    fn default() -> Self {
        Self {
            points: Default::default(),
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
            }
        }
    }
    if *wave_now {
        stats.time += time.delta()
    }

    deaths.iter_entities(&mut diers, |_, points| {
        stats.player.points += points.value;
    });
}
