use crate::common::*;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Editor>().add_system(editor.exclusive_system());
    }
}

#[derive(Default)]
pub struct Editor {
    pub enabled: bool,
    show_menu: bool,
}

fn editor(mut ctx: ResMut<EguiContext>, mut editor: ResMut<Editor>) {

}
