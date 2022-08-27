use crate::common::*;

/// Resource
#[derive(Default)]
pub struct TheFont {
    pub font: Handle<Font>,
}

/// Note that changes to this are ignored
#[derive(Component)]
pub struct WorldText {
    pub text: Vec<(String, Color)>,
    pub size: f32,
}

//

pub struct HudElementsPlugin;

impl Plugin for HudElementsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TheFont>()
            .add_startup_system(load_the_font)
            .add_system(spawn_world_text);
    }
}

fn load_the_font(mut font: ResMut<TheFont>, server: Res<AssetServer>) {
    font.font = server.load("Inconsolata-Bold.ttf");
}

fn spawn_world_text(
    mut commands: Commands, text: Query<(Entity, &WorldText), Added<WorldText>>, font: Res<TheFont>,
) {
    let real_text_size = 32.;

    for (entity, text) in text.iter() {
        commands.entity(entity).with_children(|parent| {
            parent
                .spawn_bundle(Text2dBundle {
                    text: Text {
                        sections: text
                            .text
                            .iter()
                            .map(|(string, color)| TextSection {
                                value: string.clone(),
                                style: TextStyle {
                                    font: font.font.clone(),
                                    font_size: real_text_size,
                                    color: *color,
                                },
                            })
                            .collect(),
                        alignment: TextAlignment::CENTER,
                    },
                    transform: Transform::from_scale(Vec3::new(
                        text.size / real_text_size,
                        text.size / real_text_size,
                        1.,
                    )),
                    ..default()
                })
                .insert(Depth::WorldText);
        });
    }
}
