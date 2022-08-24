use crate::{common::*, present::camera::WindowInfo};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(menu);
    }
}

#[derive(Default)]
struct DebugMenu {
    show: bool,
}

#[derive(Component)]
struct SingleFrameEntity;

fn menu(
    mut ctx: ResMut<EguiContext>, keys: Res<Input<KeyCode>>, mut menu: Local<DebugMenu>,
    window: Res<WindowInfo>, mut commands: Commands,
    delete_us: Query<Entity, With<SingleFrameEntity>>,
    mut windows: ResMut<Windows>
) {
    if keys.just_pressed(KeyCode::F2) {
        menu.show.flip();
    }
    if keys.just_pressed(KeyCode::F3) {
        use bevy::window::WindowMode::*;
        let window = windows.primary_mut();
        let make_fullscreen = match window.mode() {
            Windowed => true,
            BorderlessFullscreen | SizedFullscreen | Fullscreen => false,
        };
        window.set_mode(if make_fullscreen {
            BorderlessFullscreen
        } else {
            Windowed
        });
    }
    if menu.show {
        ctx.popup("debug::menu", vec2(-1., 1.), true, |ui| {
            ui.label(format!(
                "Window {} px, {} world",
                window.size,
                window.world_size()
            ));
        });
        use bevy_lyon::*;
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shapes::Circle {
                    radius: 2.,
                    ..default()
                },
                DrawMode::Fill(FillMode::color(Color::GREEN)),
                Transform::from_translation(window.cursor.extend(999.)),
            ))
            .insert(SingleFrameEntity);
    }
    for entity in delete_us.iter() {
        commands.entity(entity).despawn_recursive()
    }
}
