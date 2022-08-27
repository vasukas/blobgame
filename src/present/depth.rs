use crate::common::*;

/// Note that this can't be changed after initial creation
#[derive(Component, Clone, Copy)]
pub enum Depth {
    Light,
    Player,
    Wall,
    Projectile,
    Effect,
    ImportantEffect,
}

impl Depth {
    fn z_exact(self) -> f32 {
        match self {
            Depth::Light => 110.,
            Depth::Player => 300.,
            Depth::Projectile => 500.,
            Depth::Wall => 800.,
            Depth::Effect => 900.,
            Depth::ImportantEffect => 950.,
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
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            set_depth.before(bevy::transform::TransformSystem::TransformPropagate),
        );
    }
}

fn set_depth(
    mut entities: Query<(&mut Transform, &Depth, Option<&Parent>), Added<Depth>>,
    parents: Query<(&Depth, Option<&Parent>)>,
) {
    for (mut transform, depth, parent) in entities.iter_mut() {
        let mut z = depth.z_fuzzy();

        parent.map(|p| p.get()).while_some(|parent| {
            parents.get(parent).ok().and_then(|(depth, next_parent)| {
                z -= depth.z_exact();
                next_parent.map(|p| p.get())
            })
        });

        transform.translation.z = z;
    }
}
