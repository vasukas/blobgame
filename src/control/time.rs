use crate::common::*;

/// Resource - gameplay time
pub struct GameTime {
    now: Duration,
    delta: Duration,

    /// How fast time advances - set this to value >= 0 to slow down or fasten flow of gameplay-related time
    pub scale: f32,
}

impl GameTime {
    pub fn _now(&self) -> Duration {
        self.now
    }

    /// Note that this might be zero!
    pub fn _delta_seconds(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    pub fn reached(&self, time: Duration) -> bool {
        self.now >= time
    }

    pub fn _passed(&self, since: Duration) -> Duration {
        self.now.checked_sub(since).unwrap_or_default()
    }
}

//

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTime {
            now: default(),
            delta: Duration::from_secs_f32(1. / 60.),
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

    // TODO: which mode to use? maybe change time_scale instead of dt?
    physics.timestep_mode = TimestepMode::Interpolated {
        dt: delta.as_secs_f32(),
        time_scale: 1.,
        substeps: 1,
    };
}
