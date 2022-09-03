use crate::common::*;

/// Resource - gameplay time
pub struct GameTime {
    now: Duration,
    delta: Duration,

    /// How fast time advances, where 1 is normal and 0 is complete stop
    pub scale: f32,
}

impl GameTime {
    /// Current time. Monotonically advances
    pub fn now(&self) -> Duration {
        self.now
    }

    /// How much time advanced last frame
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// Note that this might be zero!
    pub fn delta_seconds(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    /// Have reached or passed that time
    pub fn reached(&self, time: Duration) -> bool {
        self.now >= time
    }

    /// How much passed since that time
    pub fn passed(&self, since: Duration) -> Duration {
        self.now.checked_sub(since).unwrap_or_default()
    }

    /// Returns `passed(since) / period`
    pub fn t_passed(&self, since: Duration, period: Duration) -> f32 {
        self.passed(since).as_secs_f32() / period.as_secs_f32()
    }
}

//

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTime {
            now: default(),
            delta: default(),
            scale: 1.,
        })
        .add_system_to_stage(CoreStage::PreUpdate, advance_time);
    }
}

fn advance_time(
    time: Res<Time>, mut game_time: ResMut<GameTime>, mut physics: ResMut<RapierConfiguration>,
) {
    let delta = time.delta().mul_f32(game_time.scale);
    game_time.delta = delta;
    game_time.now += delta;

    physics.timestep_mode = TimestepMode::Interpolated {
        // rapier hangs if dt is zero
        dt: time.delta_seconds(),
        time_scale: game_time.scale,
        substeps: 1,
    };
}
