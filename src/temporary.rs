use crate::{common::*, present::camera::WorldCamera};

pub struct TemporaryPlugin;

impl Plugin for TemporaryPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(tmp_keys);
    }
}

fn tmp_keys(
    mut camera: Query<(&mut WorldCamera, &mut Transform)>, keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if keys.just_pressed(KeyCode::Key1) {
        camera.single_mut().0.target_size *= 2.
    }
    if keys.just_pressed(KeyCode::Key2) {
        camera.single_mut().0.target_size /= 2.
    }

    let speed = 40. * time.delta_seconds();
    if keys.pressed(KeyCode::Left) {
        camera.single_mut().1.translation.x -= speed
    }
    if keys.pressed(KeyCode::Right) {
        camera.single_mut().1.translation.x += speed
    }
    if keys.pressed(KeyCode::Up) {
        camera.single_mut().1.translation.y += speed
    }
    if keys.pressed(KeyCode::Down) {
        camera.single_mut().1.translation.y -= speed
    }
}
