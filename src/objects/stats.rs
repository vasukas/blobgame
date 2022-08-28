use super::{loot::CraftPart, spawn::WaveEvent, weapon::CraftedWeapon};
use crate::{common::*, mechanics::health::DeathEvent};
use enum_map::EnumMap;

#[derive(Default)]
pub struct Stats {
    pub wave: usize,
    pub time: Duration,
    pub restarts: usize,

    pub player: PersistentPlayer,
    last_wave: PersistentPlayer,
}

/// Stuff restored after respawn
#[derive(Clone)]
pub struct PersistentPlayer {
    pub points: usize,
    pub craft_parts: EnumMap<CraftPart, usize>, // count
    pub weapon0: Option<(CraftedWeapon, f32)>,  // usage left
    pub weapon1: Option<(CraftedWeapon, f32)>,
}

impl Default for PersistentPlayer {
    fn default() -> Self {
        Self {
            points: Default::default(),
            craft_parts: enum_map::enum_map! {
                CraftPart::Generator => 2,
                CraftPart::Emitter => 2,
                CraftPart::Laser => 2,
                CraftPart::Magnet => 2,
            },
            weapon0: Default::default(),
            weapon1: Default::default(),
        }
    }
}

#[derive(Component)]
pub struct DeathPoints(pub usize);

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
            WaveEvent::Started => *wave_now = true,
            WaveEvent::Ended => {
                stats.last_wave = stats.player.clone();
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

    deaths.iter_cmd_mut(&mut diers, |_, points| stats.player.points += points.0);
}
