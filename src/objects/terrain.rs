use crate::common::*;

#[derive(Component)]
pub struct TerrainPoint {
    pub line_id: usize,
    pub point_id: usize,
}

/// Event - (re)spawn all terrain
pub struct SpawnTerrain;

//

const LINE_COLOR: Color = Color::rgb(0.75, 0.75, 0.75);
const LINE_WIDTH: f32 = 0.8;

pub struct SomePlugin;

impl Plugin for SomePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnTerrain>().add_system(spawn_terrain);
    }
}

#[derive(Component)]
struct TerrainLine;

fn spawn_terrain(
    mut commands: Commands, mut cmds: EventReader<SpawnTerrain>,
    points: Query<(&GlobalTransform, &TerrainPoint)>, lines: Query<Entity, With<TerrainLine>>,
) {
    if cmds.iter().count() != 0 {
        for entity in lines.iter() {
            commands.entity(entity).despawn_recursive()
        }

        let mut lines: HashMap<_, Vec<_>> = HashMap::new();
        for (transform, point) in points.iter() {
            lines
                .entry(point.line_id)
                .or_default()
                .push((transform.pos_2d(), point.point_id));
        }

        for (_, mut points) in lines {
            points.sort_by(|a, b| a.1.cmp(&b.1));
            let points: Vec<_> = points.iter().map(|v| v.0).collect();

            use bevy_lyon::*;
            commands
                .spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Polygon {
                        points: points.clone(),
                        closed: false,
                    },
                    DrawMode::Stroke(StrokeMode::new(LINE_COLOR, LINE_WIDTH)),
                    default(),
                ))
                /* .insert(Depth::WallTile) */
                //
                .insert(GameplayObject)
                .insert(TerrainLine)
                //
                .insert(RigidBody::KinematicPositionBased)
                .insert(PhysicsType::Obstacle.rapier())
                .insert(Collider::polyline(points, None));
        }
    }
}
