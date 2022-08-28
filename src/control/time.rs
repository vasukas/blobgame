use super::input::InputLock;
use crate::common::*;

/// Resource - gameplay time
pub struct GameTime {
    now: Duration,
    delta: Duration,
}

impl GameTime {
    pub fn now(&self) -> Duration {
        self.now
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// Note that this might be zero!
    pub fn delta_seconds(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    pub fn reached(&self, time: Duration) -> bool {
        self.now >= time
    }

    pub fn passed(&self, since: Duration) -> Duration {
        self.now.checked_sub(since).unwrap_or_default()
    }

    pub fn t_passed(&self, since: Duration, period: Duration) -> f32 {
        self.passed(since).as_secs_f32() / period.as_secs_f32()
    }
}

#[derive(Component, Default)]
pub struct TimeMode {
    pub main_menu: bool,
    pub craft_menu: bool,
    pub player_alive: bool,
}

//

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTime {
            now: default(),
            delta: Duration::from_secs_f32(1. / 60.),
        })
        .init_resource::<TimeMode>()
        .add_system_to_stage(CoreStage::PreUpdate, advance_time);
    }
}

fn advance_time(
    time: Res<Time>, mut game_time: ResMut<GameTime>, mut physics: ResMut<RapierConfiguration>,
    mode: Res<TimeMode>, mut input_lock: ResMut<InputLock>,
) {
    // TODO: REMOVE THIS FROM HERE
    let scale = if mode.main_menu || mode.craft_menu || !mode.player_alive { 0. } else { 1. };
    input_lock.active = mode.main_menu || mode.craft_menu;
    input_lock.allow_craft = mode.craft_menu;

    let delta = time.delta().mul_f32(scale);
    game_time.delta = delta;
    game_time.now += delta;

    physics.timestep_mode = TimestepMode::Interpolated {
        dt: time.delta_seconds(),
        time_scale: scale,
        substeps: 1,
    };
}
