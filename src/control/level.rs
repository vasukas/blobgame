use crate::{common::*, objects::spawn::*};
use bevy::{asset::AssetLoader, reflect::TypeUuid};

/// Entity must be deleted on level despawn
#[derive(Component)]
pub struct GameplayObject;

/// Event (command)
pub enum LevelCommand {
    Load(Handle<Level>),
    Unload,
    Reload,
}

/// Event (not sent on reloading)
pub enum LevelEvent {
    /// Loaded and spawned new level
    Loaded {
        title: String,
    },
    /// Unloaded all levels (exited from game)
    Unloaded,
    Reloaded,
}

//

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LevelCommand>()
            .add_event::<LevelEvent>()
            .add_asset::<Level>()
            .init_asset_loader::<LevelAssetLoader>()
            .init_resource::<CurrentLevel>()
            .add_system_to_stage(CoreStage::First, load_level.exclusive_system().at_start());
    }
}

#[derive(Default)]
struct CurrentLevel {
    old: Handle<Level>,
    level: Handle<Level>,
    loaded: bool,
}

fn load_level(
    mut cmds: EventReader<LevelCommand>, mut current: ResMut<CurrentLevel>, mut commands: Commands,
    mut events: EventWriter<LevelEvent>, levels: Res<Assets<Level>>,
    objects: Query<Entity, With<GameplayObject>>,
) {
    // process commands
    if let Some(cmd) = cmds.iter().last() {
        // despawn old objects
        for entity in objects.iter() {
            commands.entity(entity).despawn_recursive()
        }

        // prepare loading
        match cmd {
            LevelCommand::Load(level) => {
                log::info!("Level: start load {:?}", level);
                *current = CurrentLevel {
                    level: level.clone(),
                    ..default()
                };
            }
            LevelCommand::Unload => {
                log::info!("Level: unloaded");
                current.loaded = true;
                events.send(LevelEvent::Unloaded);
            }
            LevelCommand::Reload => {
                log::info!("Level: start reload");
                current.loaded = false;
                events.send(LevelEvent::Reloaded);
            }
        }
    }

    // load level if needed
    if !current.loaded {
        if let Some(level) = levels.get(&current.level) {
            current.loaded = true;
            if current.old != current.level {
                current.old = current.level.clone();
                log::info!("Level: loading {:?} - \"{}\"", current.level, level.title);
            }
            events.send(LevelEvent::Loaded {
                title: level.title.clone(),
            });

            for (pos, ty) in &level.areas {
                commands
                    .spawn_bundle(SpatialBundle::default())
                    .insert(GameplayObject)
                    .insert(ty.clone())
                    .insert(LevelAreaPolygon(pos.clone()));
            }
            for (pos, ty) in &level.points {
                commands
                    .spawn_bundle(SpatialBundle {
                        transform: Transform::from_translation(pos.extend(0.)),
                        ..default()
                    })
                    .insert(GameplayObject)
                    .insert(ty.clone());
            }
        }
    }
}

#[derive(Default, TypeUuid)]
#[uuid = "a8bdb05c-e32a-4cec-aa3f-f7b9bd5168d1"]
pub struct Level {
    title: String,
    /// List of polygons. If polygon is closed, first and last points are equal
    areas: Vec<(Vec<Vec2>, LevelArea)>,
    points: Vec<(Vec2, LevelObject)>,
}

impl Level {
    fn from_svg_bytes(bytes: &[u8]) -> anyhow::Result<Level> {
        let mut svg = crate::utils::svg::File::from_bytes(bytes)?;
        svg.fix();

        let mut level = Level::default();
        level.title = svg.title.clone().unwrap_or_else(|| "UNKNOWN".to_string());

        for point in svg.points {
            match LevelObject::from_id(&point.id) {
                Ok(Some(obj)) => level.points.push((point.pos, obj)),
                Ok(None) => log::warn!("SVG: invalid point id \"{}\"", point.id),
                Err(error) => {
                    anyhow::bail!("SVG: failed to parse point id \"{}\": {}", point.id, error)
                }
            }
        }

        for line in svg.lines {
            match LevelArea::from_id(&line.id) {
                Ok(Some(obj)) => level.areas.push((line.pos, obj)),
                Ok(None) => log::warn!("SVG: invalid line id \"{}\"", line.id),
                Err(error) => {
                    anyhow::bail!("SVG: failed to parse line id \"{}\": {}", line.id, error)
                }
            }
        }

        Ok(level)
    }
}

#[derive(Default)]
struct LevelAssetLoader;

impl AssetLoader for LevelAssetLoader {
    fn load<'a>(
        &'a self, bytes: &'a [u8], load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            Level::from_svg_bytes(bytes).map(|level| {
                load_context.set_default_asset(bevy::asset::LoadedAsset::new(level));
                ()
            })
        })
    }
    fn extensions(&self) -> &[&str] {
        &["svg"]
    }
}
