use crate::common::*;

#[derive(Component, Clone, Copy)]
pub enum Depth {
    Light,
    BackgroundObject,
    TerrainPolygon,
    TerrainOutline,
}

impl Depth {
    fn z_exact(self) -> f32 {
        match self {
            Depth::Light => 110.,
            Depth::BackgroundObject => 100.,
            Depth::TerrainPolygon => 900.,
            Depth::TerrainOutline => 910.,
        }
    }

    fn z_fuzzy(self) -> f32 {
        use rand::*;
        self.z_exact() + thread_rng().gen_range(0. ..1.)
    }
}

/// Use this for children of entities which have their own Depth.
/// Dunno if there is a better solution.
#[derive(Component)]
pub struct ChildDepth {
    pub parent: Option<Depth>,
    pub this: Depth,
}

impl ChildDepth {
    pub fn new(parent: Option<&Depth>, this: Depth) -> Self {
        Self {
            parent: parent.copied(),
            this,
        }
    }
}

//

pub struct DepthPlugin;

impl Plugin for DepthPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            set_depth.before(bevy::transform::TransformSystem::TransformPropagate),
        );
    }
}

fn set_depth(
    mut entities: Query<(&mut Transform, &Depth), Added<Depth>>,
    mut relatives: Query<(&mut Transform, &ChildDepth), (Added<ChildDepth>, Without<Depth>)>,
) {
    // TODO: make this work with hierarchies somehow - and allow depth change
    for (mut transform, depth) in entities.iter_mut() {
        transform.translation.z = depth.z_fuzzy();
    }
    for (mut transform, depth) in relatives.iter_mut() {
        transform.translation.z =
            depth.this.z_fuzzy() - depth.parent.map(|d| d.z_exact()).unwrap_or_default();
    }
}
