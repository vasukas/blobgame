use crate::common::*;

#[derive(Component, Clone, Copy)]
pub enum Depth {
    TerrainPolygon,
    TerrainOutline,
}

impl Depth {
    fn z_exact(self) -> f32 {
        match self {
            Depth::TerrainPolygon => 900.,
            Depth::TerrainOutline => 910.,
        }
    }

    fn z_fuzzy(self) -> f32 {
        use rand::*;
        self.z_exact() + thread_rng().gen_range(0. ..1.)
    }
}

//

pub struct DepthPlugin;

impl Plugin for DepthPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, set_depth);
    }
}

fn set_depth(mut entities: Query<(&mut Transform, &Depth), Changed<Depth>>) {
    for (mut transform, depth) in entities.iter_mut() {
        transform.translation.z = depth.z_fuzzy();
    }
}
