use crate::common::*;

#[derive(Default)]
pub struct Stats {
    pub wave: usize,
}

//

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stats>();
    }
}
