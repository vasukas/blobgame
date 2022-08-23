use crate::common::*;
use std::sync::Arc;

pub type ImageVec = Arc<Vec<Handle<Image>>>;

/// Requires SpatialBundle
#[derive(Component, Default)]
pub struct SimpleSprite {
    pub images: ImageVec,
    pub frame_duration: Duration,

    pub color: Color,
    pub size: Vec2,

    // state
    pub frame: usize,
    pub until: Duration,
}

//

pub struct SimpleSpritePlugin;

impl Plugin for SimpleSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_sprite).add_system(update_sprite);
    }
}

fn spawn_sprite(
    mut commands: Commands, mut sprites: Query<(Entity, &mut SimpleSprite), Added<SimpleSprite>>,
) {
    for (entity, mut sprite) in sprites.iter_mut() {
        if let Some((frame, image)) = sprite.images.iter().enumerate().get_random_select() {
            commands
                .entity(entity)
                .insert(Sprite {
                    custom_size: Some(Vec2::ZERO),
                    ..default()
                })
                .insert(image.clone());
            sprite.frame = frame;
        }
    }
}

fn update_sprite(
    mut sprites: Query<(&mut SimpleSprite, &mut Sprite, &mut Handle<Image>)>, time: Res<GameTime>,
) {
    for (mut data, mut sprite, mut image) in sprites.iter_mut() {
        if time.reached(data.until) && data.images.len() > 0 {
            data.frame = (data.frame + 1) % data.images.len();
            data.until = time.now() + data.frame_duration;
            *image = data.images.get(data.frame).unwrap().clone();
        }
        sprite.color = data.color;
        sprite.custom_size = Some(data.size);
    }
}
