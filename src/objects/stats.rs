use super::spawn::WaveEvent;
use crate::{common::*, mechanics::health::DeathEvent};

#[derive(Default)]
pub struct Stats {
    pub wave: usize,
    pub points: usize,
    pub time: Duration,
    pub restarts: usize,

    // used to restore after respawn
    last_wave_points: usize,
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
                stats.last_wave_points = stats.points;
                *wave_now = false;
            }
            WaveEvent::Restart => {
                stats.points = stats.last_wave_points;
                stats.restarts += 1;
            }
        }
    }
    if *wave_now {
        stats.time += time.delta()
    }

    deaths.iter_cmd_mut(&mut diers, |_, points| stats.points += points.0);
}
