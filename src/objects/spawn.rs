use crate::common::*;

use super::player::Player;

/// If it's closed, last point will be equal to first
#[derive(Component)]
pub struct LevelAreaPolygon(pub Vec<Vec2>);

#[derive(Component, Clone)]
pub enum LevelArea {
    Terrain,
    Checkpoint,
    LevelExit,
}

impl LevelArea {
    pub fn from_id(id: &str) -> Self {
        match id {
            "check" => Self::Checkpoint,
            "exit" => Self::LevelExit,
            _ => Self::Terrain,
        }
    }
}

#[derive(Component, Clone)]
pub enum LevelObject {
    PlayerSpawn,
}

impl LevelObject {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "player" => Some(LevelObject::PlayerSpawn),
            _ => None,
        }
    }
}

//

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        /*         app.init_resource::<GameAssets>()
        .add_system_to_stage(CoreStage::First, spawn)
        .add_startup_system(load_assets); */

        app.add_system_to_stage(CoreStage::First, spawn_areas.exclusive_system().at_end())
            .add_system_to_stage(CoreStage::First, spawn_objects.exclusive_system().at_end());
    }
}

fn spawn_areas(
    mut commands: Commands, areas: Query<(Entity, &LevelAreaPolygon, &LevelArea), Added<LevelArea>>,
) {
    use bevy_lyon::*;

    let terrain_fill_color = Color::BLACK;
    let terrain_outline_color = Color::WHITE;
    let terrain_outline_width = 0.1;

    for (entity, polygon, area) in areas.iter() {
        match area {
            LevelArea::Terrain => {
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
                                    closed: false,
                                },
                                DrawMode::Fill(FillMode::color(terrain_fill_color)),
                                default(),
                            ))
                            .insert(Depth::TerrainPolygon);
                        parent
                            .spawn_bundle(GeometryBuilder::build_as(
                                &shapes::Polygon {
                                    points: polygon.0.clone(),
                                    closed: false,
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

            LevelArea::Checkpoint => todo!(),

            LevelArea::LevelExit => todo!(),
        }
    }
}

fn spawn_objects(
    mut commands: Commands, objects: Query<(Entity, &LevelObject), Added<LevelObject>>,
) {
    for (entity, ty) in objects.iter() {
        match ty {
            LevelObject::PlayerSpawn => {
                commands.entity(entity).insert(Player::default());
            }
        }
    }
}

/*
fn spawn(
    mut commands: Commands, tiles: Query<(&GlobalTransform, &Spawn), Added<SpawnActive>>,
    assets: Res<GameAssets>,
) {
    for (transform, spawn) in tiles.iter() {
        //
    }
}

#[derive(Default)]
struct GameAssets {
    player_sprite: ImageVec,
    wall_sprite: ImageVec,
}

fn load_assets(
    mut loading: ResMut<Loading>, server: Res<AssetServer>, mut assets: ResMut<GameAssets>,
) {
    assets.player_sprite = ImageVec::new(loading.add_n(&server, "sprites/player/circ?.png", 3));
    assets.wall_sprite = ImageVec::new(loading.add_n(&server, "sprites/wall/wall?.png", 3));
}
*/
