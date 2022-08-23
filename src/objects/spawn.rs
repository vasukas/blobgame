use super::player::*;
use crate::{
    common::*,
    mechanics::{combat::ArenaDoor, physics::*},
    present::light::*,
};

/// If it's closed, last point will be equal to first
#[derive(Component)]
pub struct LevelAreaPolygon(pub Vec<Vec2>);

#[derive(Component, Clone)]
pub enum LevelArea {
    Terrain,
    Checkpoint(u64),
    LevelExit(String),
    Door,
}

impl LevelArea {
    pub fn from_id(full_id: &str) -> anyhow::Result<Option<Self>> {
        let mut split = full_id.split("_");
        let id = split.next().unwrap_or(full_id);
        let arg = split.next();

        match id {
            "check" => Ok(Some(Self::Checkpoint(parse_id_arg(arg)?))),
            "exit" => Ok(Some(Self::LevelExit(option_err(arg)?.to_string()))),
            "door" => Ok(Some(Self::Door)),
            id if id.starts_with("path") => Ok(Some(Self::Terrain)),
            _ => Ok(None),
        }
    }
}

#[derive(Component, Clone)]
pub enum LevelObject {
    PlayerSpawn(u64),
    Crystal,
    Torch,
    Spewer,
}

impl LevelObject {
    pub fn from_id(full_id: &str) -> anyhow::Result<Option<Self>> {
        let mut split = full_id.split("_");
        let id = split.next().unwrap_or(full_id);
        let arg = split.next();

        match id {
            "player" => Ok(Some(LevelObject::PlayerSpawn(parse_id_arg(arg)?))),
            "crystal" => Ok(Some(LevelObject::Crystal)),
            "torch" => Ok(Some(LevelObject::Torch)),
            "spew" => Ok(Some(LevelObject::Spewer)),
            _ => Ok(None),
        }
    }
}

fn option_err<T>(arg: Option<T>) -> anyhow::Result<T> {
    Ok(arg.ok_or_else(|| anyhow::anyhow!("No argument"))?)
}
fn parse_id_arg(arg: Option<&str>) -> anyhow::Result<u64> {
    Ok(option_err(arg)?.parse()?)
}

//

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::First, spawn_areas.exclusive_system().at_end())
            .add_system_to_stage(CoreStage::First, spawn_objects.exclusive_system().at_end());
    }
}

fn spawn_areas(
    mut commands: Commands, areas: Query<(Entity, &LevelAreaPolygon, &LevelArea), Added<LevelArea>>,
) {
    use bevy_lyon::*;

    let terrain_fill_color = Color::BLACK;
    let door_fill_color = Color::rgb(0.2, 0., 0.);
    let terrain_outline_color = Color::WHITE;
    let terrain_outline_width = 0.1;

    for (entity, polygon, area) in areas.iter() {
        match area {
            LevelArea::Terrain => {
                let closed = polygon.0.first() != polygon.0.last();
                commands
                    .entity(entity)
                    .insert(RigidBody::Fixed)
                    .insert(PhysicsType::Obstacle.rapier())
                    .insert(Collider::polyline(polygon.0.clone(), None))
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(GeometryBuilder::build_as(
                                &shapes::Polygon {
                                    points: polygon.0.clone(),
                                    closed,
                                },
                                DrawMode::Fill(FillMode::color(terrain_fill_color)),
                                default(),
                            ))
                            .insert(Depth::TerrainPolygon);
                        parent
                            .spawn_bundle(GeometryBuilder::build_as(
                                &shapes::Polygon {
                                    points: polygon.0.clone(),
                                    closed,
                                },
                                DrawMode::Stroke(StrokeMode::new(
                                    terrain_outline_color,
                                    terrain_outline_width,
                                )),
                                default(),
                            ))
                            .insert(Depth::TerrainOutline);
                    });
            }

            LevelArea::Checkpoint(id) => {
                commands
                    .entity(entity)
                    .insert(RigidBody::Fixed)
                    .insert(PhysicsType::Script.rapier())
                    .insert(convex_decomposition(&polygon.0))
                    .insert(Sensor)
                    //
                    .insert(CheckpointArea(*id))
                    .insert(CollectContacts::default());
            }

            LevelArea::LevelExit(id) => {
                commands
                    .entity(entity)
                    .insert(RigidBody::Fixed)
                    .insert(PhysicsType::Script.rapier())
                    .insert(convex_decomposition(&polygon.0))
                    .insert(Sensor)
                    //
                    .insert(ExitArea(id.clone()))
                    .insert(CollectContacts::default());
            }

            LevelArea::Door => {
                commands
                    .entity(entity)
                    // No RigidBody! doors are disabled by default
                    .insert(PhysicsType::Obstacle.rapier())
                    .insert(convex_decomposition(&polygon.0))
                    //
                    .insert(ArenaDoor)
                    .insert(Visibility { is_visible: false })
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(GeometryBuilder::build_as(
                                &shapes::Polygon {
                                    points: polygon.0.clone(),
                                    closed: false,
                                },
                                DrawMode::Fill(FillMode::color(door_fill_color)),
                                default(),
                            ))
                            .insert(Depth::TerrainPolygon);
                    });
            }
        }
    }
}

fn spawn_objects(
    mut commands: Commands, objects: Query<(Entity, &LevelObject), Added<LevelObject>>,
    assets: Res<MyAssets>,
) {
    for (entity, ty) in objects.iter() {
        match ty {
            LevelObject::PlayerSpawn(id) => {
                commands.entity(entity).insert(CheckpointSpawn(*id));
            }
            LevelObject::Crystal => {
                let size = vec2(0.6, 1.);
                commands
                    .entity(entity)
                    .insert(Sprite {
                        custom_size: Some(size),
                        ..default()
                    })
                    .insert(assets.crystal.clone())
                    .insert(Depth::BackgroundObject)
                    .insert(Light::default())
                    .insert(LightPulse {
                        period: Duration::from_secs(4),
                        source: Light {
                            radius: 10.,
                            color: Color::CYAN.with_a(0.1),
                        },
                        ..default()
                    })
                    //
                    .insert(PlunkDown {
                        distance: size.y / 2.,
                    });
            }

            LevelObject::Torch => (),

            LevelObject::Spewer => (),
        }
    }
}

/// Converts polygon into shape. Veeeery slooooooowlyeeeee.
/// Also it may panic in some cases, not sure which.
fn convex_decomposition(points: &[Vec2]) -> Collider {
    let points = if !points.is_empty() && points.first() == points.last() {
        // see below why this is needed
        &points[..points.len() - 1]
    } else {
        points
    };
    Collider::convex_decomposition(
        points,
        &points
            .iter()
            .enumerate()
            // this assumes polygon is not closed, that's why there is that thing above
            .map(|v| [v.0 as u32, ((v.0 + 1) % points.len()) as u32])
            .collect::<Vec<_>>(),
    )
}
