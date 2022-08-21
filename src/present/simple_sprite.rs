use crate::common::*;

/// Requires SpatialBundle
#[derive(Component, Default)]
pub struct SimpleSprite {
    pub images: Vec<Handle<Image>>,
    pub frame: Duration,

    pub color: Color,
    pub size: Vec2,
}

//

pub struct SimpleSpritePlugin;

impl Plugin for SimpleSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_sprite)
            .add_system(update_image)
            .add_system(update_sprite);
    }
}

#[derive(Component)]
struct SpriteState {
    frame: usize,
    until: Duration,
}

fn spawn_sprite(
    mut commands: Commands, sprites: Query<(Entity, &SimpleSprite), Added<SimpleSprite>>,
) {
    for (entity, sprite) in sprites.iter() {
        if let Some((frame, image)) = sprite.images.iter().enumerate().get_random_select() {
            commands
                .entity(entity)
                .insert(SpriteState {
                    frame,
                    until: default(),
                })
                .insert(Sprite {
                    color: sprite.color,
                    custom_size: Some(sprite.size),
                    ..default()
                })
                .insert(image.clone());
        }
    }
}

fn update_image(
    mut sprites: Query<(&SimpleSprite, &mut SpriteState, &mut Handle<Image>)>, time: Res<GameTime>,
) {
    for (data, mut state, mut image) in sprites.iter_mut() {
        if time.reached(state.until) && data.images.len() > 0 {
            state.frame = (state.frame + 1) % data.images.len();
            *image = data.images.get(state.frame).unwrap().clone();
        }
    }
}

fn update_sprite(mut sprites: Query<(&SimpleSprite, &mut Sprite), Changed<SimpleSprite>>) {
    for (data, mut sprite) in sprites.iter_mut() {
        sprite.color = data.color;
        sprite.custom_size = Some(data.size);
    }
}
