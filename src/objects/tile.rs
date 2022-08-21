use crate::common::*;

#[derive(Component)]
pub struct Tile;

//

pub struct SomePlugin;

impl Plugin for SomePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_tiles);
    }
}

fn spawn_tiles(mut commands: Commands, tiles: Query<Entity, Added<Tile>>) {
    for entity in tiles.iter() {
        commands
            .entity(entity)
            .insert(RigidBody::KinematicPositionBased)
            .insert(PhysicsType::Obstacle.rapier())
            .insert(Collider::cuboid(TILE_SIZE / 2., TILE_SIZE / 2.));
    }
}
