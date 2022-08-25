use crate::common::*;
use bevy_kira_audio::prelude::*;

use super::camera::WindowInfo;

/// Event
#[derive(Default)]
pub struct Sound {
    pub sound: Handle<AudioSource>,
    pub position: Option<Vec2>,
}

#[derive(Component)]
pub struct AudioListener;

//

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_kira_audio::AudioPlugin)
            .init_resource::<SoundCollection>()
            .init_resource::<ListenerConfig>()
            .add_event::<Sound>()
            .add_system(apply_settings)
            .add_system(update_listener_config)
            .add_system(play_sounds)
            .add_system(update_positional.exclusive_system().at_start());
    }
}

/// Keep sounds loaded; this is a hack
#[derive(Default)]
struct SoundCollection {
    sounds: HashMap<bevy::asset::HandleId, Handle<AudioSource>>,
}

impl SoundCollection {
    fn add(&mut self, sound: &Handle<AudioSource>) {
        self.sounds.entry(sound.id).or_insert_with(|| sound.clone());
    }
}

#[derive(Default)]
struct ListenerConfig {
    pos: Vec2,
    size: Vec2,
}

impl ListenerConfig {
    /// Returns (volume, panning)
    fn calculate(&self, pos: Vec2, _relative_range: f32) -> (f64, f64) {
        let min_distance = self.size.max_element() * 0.08;
        let max_distance = self.size.max_element() * 1.2;
        let min_panning_distance = self.size.max_element() * 0.1;
        let max_panning_distance = self.size.max_element() * 0.8;
        let max_panning = 1.;

        let delta = pos - self.pos;
        let distance = delta.length();

        let t_distance = inverse_lerp(distance, min_distance, max_distance).clamp(0., 1.);
        let volume = 1. - t_distance;

        // TODO: fix, invalid
        // let t_panning =
        //     inverse_lerp(min_panning_distance, max_panning_distance, distance).clamp(0., 1.);
        // let kxy = (delta / distance.max(0.1)).dot(Vec2::X);
        // let pan = kxy.abs().min(max_panning).copysign(kxy);

        // (volume as f64, (pan * t_panning * 0.5 + 0.5) as f64)
        (volume as f64, 0.5)
    }
}

#[derive(Component)]
struct PositionalSound {
    handle: Handle<AudioInstance>,
}

//

fn apply_settings(audio: Res<Audio>, settings: Res<Settings>) {
    if settings.is_added() || settings.is_changed() {
        audio.set_volume(settings.master_volume as f64);
    }
}

fn update_listener_config(
    mut config: ResMut<ListenerConfig>, listeners: Query<&GlobalTransform, With<AudioListener>>,
    window: Res<WindowInfo>,
) {
    if let Ok(pos) = listeners.get_single() {
        config.pos = pos.pos_2d()
    }
    config.size = window.world_size()
}

fn play_sounds(
    mut events: EventReader<Sound>, audio: Res<Audio>, mut collection: ResMut<SoundCollection>,
    config: Res<ListenerConfig>, mut commands: Commands,
) {
    for event in events.iter() {
        collection.add(&event.sound);

        use rand::*;
        let (volume, panning) = event
            .position
            .map(|pos| config.calculate(pos, 1.))
            .unwrap_or((1., 0.));

        let mut cmd = audio.play(event.sound.clone());
        cmd.with_playback_rate(thread_rng().gen_range(0.9..1.2))
            .with_volume(volume)
            .with_panning(panning);
        if let Some(pos) = event.position {
            commands
                .spawn_bundle(SpatialBundle::from_transform(Transform::new_2d(pos)))
                .insert(PositionalSound {
                    handle: cmd.handle(),
                });
        }
    }
}

fn update_positional(
    mut commands: Commands, sounds: Query<(Entity, &GlobalTransform, &PositionalSound)>,
    config: Res<ListenerConfig>, mut instances: ResMut<Assets<AudioInstance>>, time: Res<Time>,
) {
    let tween = || AudioTween::linear(time.delta());
    for (entity, pos, sound) in sounds.iter() {
        if let Some(instance) = instances.get_mut(&sound.handle) {
            if instance.state() == PlaybackState::Stopped {
                commands.entity(entity).despawn_recursive()
            } else {
                let (volume, panning) = config.calculate(pos.pos_2d(), 1.);
                instance.set_volume(volume, tween());
                instance.set_panning(panning, tween());
            }
        } else {
            // it doesn't exist if sound source wasn't loaded yet. which is entire other problem...
            // log::warn!("AudioInstance doesn't exist, this shouldn't happen");
            // commands.entity(entity).despawn_recursive()
        }
    }
}
