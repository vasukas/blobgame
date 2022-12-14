use super::camera::WindowInfo;
use crate::{common::*, control::time::TimeMode, objects::spawn::SpawnControl};
use bevy_kira_audio::prelude::*;

/// Event
#[derive(Default)]
pub struct Sound {
    pub sound: Handle<AudioSource>,
    pub position: Option<Vec2>,
    pub non_randomized: bool,
}

#[derive(Component)]
pub struct AudioListener;

/// Resource
#[derive(Default)]
pub struct Beats {
    // settings
    pub level: usize,

    // report
    pub start: Option<Duration>,
    pub period: Duration,
    pub count: i32,
}

impl Beats {
    pub fn in_beat(&self, time: &Time) -> bool {
        let allow_before = 0.12;
        let allow_after = 0.13;

        match self.start {
            Some(start) => {
                let period = self.period.as_secs_f32();
                let at = (time.time_since_startup() - start).as_secs_f32() % period;
                at < allow_after || at > period - allow_before
            }
            None => false,
        }
    }
}

//

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_kira_audio::AudioPlugin)
            .init_resource::<ListenerConfig>()
            .init_resource::<Beats>()
            .add_event::<Sound>()
            .add_system(apply_settings)
            .add_system(update_listener_config)
            .add_system(play_sounds)
            .add_system(update_positional.exclusive_system().at_start())
            .add_system(menu_drone)
            .add_system_to_stage(CoreStage::First, beats);
    }
}

// base impl

const K_VOLUME: f64 = 0.25; // since apparently all sound assets are normalized

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

        let t_panning =
            inverse_lerp(distance, min_panning_distance, max_panning_distance).clamp(0., 1.);
        let kxy = (delta / distance.max(0.1)).dot(Vec2::X);
        let pan = kxy.abs().min(max_panning).copysign(kxy);

        (volume as f64, (pan * t_panning * 0.5 + 0.5) as f64)
    }

    // seconds
    fn startup_delay(&self, pos: Vec2) -> f64 {
        let speed_of_sound = self.size.max_element() * 4.;
        let distance = (pos - self.pos).length();
        (distance / speed_of_sound) as f64
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
    mut events: EventReader<Sound>, audio: Res<Audio>, config: Res<ListenerConfig>,
    mut commands: Commands, time_mode: Res<TimeMode>,
) {
    let leading_silence = 0.25; // TODO: this is atrocious hack since bevy_kira_audio doesn't expose kira's start time

    for event in events.iter() {
        use rand::*;
        let ((volume, panning), start_pos) = event
            .position
            .map(|pos| {
                (
                    config.calculate(pos, 1.),
                    (leading_silence - config.startup_delay(pos)).max(0.),
                )
            })
            .unwrap_or(((1., 0.5), 0.));

        let mut cmd = audio.play(event.sound.clone());
        if !event.non_randomized {
            cmd.with_playback_rate(
                thread_rng().gen_range(0.9..1.2) * time_mode.overriden.unwrap_or(1.) as f64,
            );
        }
        cmd.with_volume(volume * K_VOLUME)
            .with_panning(panning)
            .start_from(start_pos);
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
                instance.set_volume(volume * K_VOLUME, tween());
                instance.set_panning(panning, tween());
            }
        } else {
            // it doesn't exist if sound source wasn't loaded yet. which is entire other problem...
            // log::warn!("AudioInstance doesn't exist, this shouldn't happen");
            // commands.entity(entity).despawn_recursive()
        }
    }
}

// gameplay-related stuff

fn menu_drone(
    audio: Res<Audio>, mut sound: Local<Option<Handle<AudioInstance>>>,
    mut instances: ResMut<Assets<AudioInstance>>, spawn: Res<SpawnControl>, assets: Res<MyAssets>,
) {
    match sound.as_ref() {
        Some(sound) => {
            if spawn.is_game_running() {
                if let Some(sound) = instances.get_mut(sound) {
                    sound.stop(AudioTween::linear(Duration::from_secs(1)));
                }
            }
        }
        None => {
            if !spawn.is_game_running() {
                *sound = Some(audio.play(assets.ui_menu_drone.clone()).looped().handle())
            }
        }
    }
}

fn beats(
    mut beats: ResMut<Beats>, time: Res<Time>, time_mode: Res<TimeMode>, audio: Res<Audio>,
    assets: Res<MyAssets>,
) {
    if beats.level != 0 && !time_mode.stopped() {
        beats.period = match beats.level {
            1 => Duration::from_millis(1000),
            _ => Duration::from_millis(500),
        };
        let initial_delay = Duration::from_millis(300);

        let start = *beats
            .start
            .get_or_insert(time.time_since_startup() + initial_delay);
        match time.time_since_startup().checked_sub(start) {
            Some(passed) => {
                let count = (passed.as_micros() / beats.period.as_micros()) as i32;
                if count != beats.count {
                    beats.count = count;
                    audio.play(assets.beat.clone());
                }
            }
            None => (),
        }
    } else {
        beats.start = None;
        beats.count = 0;
    }
}
