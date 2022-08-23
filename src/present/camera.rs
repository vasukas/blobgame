use crate::common::*;

/// Resource - info about main window
#[derive(Default)]
pub struct WindowInfo {
    /// World position
    pub cursor: Vec2,

    /// Visible world boundary
    pub world_min: Vec2,
    pub world_max: Vec2,

    /// Pixels, unscaled
    pub size: Vec2,
    /// Calculated scale
    pub scale: f32,
}

impl WindowInfo {
    // pub fn world_size(&self) -> Vec2 {
    //     self.world_max - self.world_min
    // }
    //     pub fn point_visible(&self, point: Vec2, size: f32) -> bool {
    //         point.in_bounds(self.world_min - size, self.world_max + size)
    //     }
}

#[derive(Component)]
pub struct WorldCamera {
    /// Minimal dimensions of visible space.
    /// Expected to be non-zero.
    pub target_size: Vec2,
}

/// Object which camera tries to follow
#[derive(Component)]
pub struct WorldCameraTarget;

//

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowInfo>()
            .add_startup_system(spawn_camera)
            .add_system(update_camera_scale)
            .add_system_to_stage(CoreStage::PreUpdate, update_window_info)
            .add_system(follow_target);
    }
}

fn spawn_camera(mut commands: Commands) {
    let target_size = vec2(40., 1.);
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(WorldCamera { target_size });
}

fn update_camera_scale(
    mut camera: Query<(&mut OrthographicProjection, &WorldCamera)>, mut info: ResMut<WindowInfo>,
) {
    if let Ok((mut projection, wcam)) = camera.get_single_mut() {
        let scale = (wcam.target_size / info.size).max_element();
        info.scale = scale;
        projection.scale = scale;
    }
}

fn update_window_info(
    windows: Res<Windows>, camera: Query<(&Camera, &GlobalTransform), With<WorldCamera>>,
    mut info: ResMut<WindowInfo>,
) {
    let (camera, camera_transform) = camera.single();
    let window = windows.primary();
    let window_size = Vec2::new(window.width().into(), window.height().into());

    if let Some(screen_pos) = window.cursor_position() {
        let ndc = (screen_pos / window_size) * 2. - 1.;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.));
        info.cursor = world_pos.truncate();
    }

    let pos = camera_transform.pos_2d();
    let halfsize = window_size * info.scale / 2.;
    info.world_min = pos - halfsize;
    info.world_max = pos + halfsize;

    info.size = window_size;
}

fn follow_target(
    mut camera: Query<(&mut Transform, &WorldCamera)>,
    target: Query<&GlobalTransform, (With<WorldCameraTarget>, Changed<GlobalTransform>)>,
    time: Res<Time>,
) {
    if let Ok((mut camera, camera_params)) = camera.get_single_mut() {
        if let Some(sum) = target.iter().map(|t| t.pos_2d()).reduce(|acc, t| acc + t) {
            let target = sum / target.iter().count() as f32;

            let delta = target - camera.pos_2d();
            let distance = delta.length_squared();

            if distance < 1. || distance > camera_params.target_size.max_element().powi(2) {
                camera.set_2d(target);
            } else {
                let magic = if distance < 10. { 0.15 } else { 0.1 } / (1. / 60.);

                let delta = delta * magic * time.delta_seconds();
                camera.translation += delta.extend(0.);
            }
        };
    }
}
