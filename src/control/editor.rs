use super::level::LevelObject;
use crate::{common::*, present::camera::WindowInfo};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Editor>().add_system(editor);
    }
}

pub struct Editor {
    pub enabled: bool,
    show_menu: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            enabled: false,
            show_menu: true,
        }
    }
}

fn editor(
    mut ctx: ResMut<EguiContext>, mut editor: ResMut<Editor>, window: Res<WindowInfo>,
    keys: Res<Input<KeyCode>>, buttons: Res<Input<MouseButton>>,
    objects: Query<(Entity, &GlobalTransform), With<LevelObject>>,
) {
    if !editor.enabled {
        return;
    }
    if keys.just_pressed(KeyCode::F2) {
        editor.show_menu.flip();
    }
    if editor.show_menu {
        //
    } else {
        if buttons.just_pressed(MouseButton::Left) {
            //
        }
        if buttons.just_pressed(MouseButton::Right) {
            //
        }
    }
}
