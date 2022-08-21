use crate::{common::*, control::loading::Loading, present::simple_sprite::SimpleSprite};

#[derive(Component, Clone, Serialize, Deserialize)]
pub enum Spawn {
    Tile { set: SpriteSet },
}

#[derive(Component)]
pub struct SpawnActive;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SpriteSet {
    Player,
    Wall,
}

//

pub struct SomePlugin;

impl Plugin for SomePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::First, spawn)
            .add_startup_system(load_sprites);
    }
}

fn spawn(
    mut commands: Commands, tiles: Query<(Entity, &Spawn), Added<SpawnActive>>,
    assets: Res<SpriteSets>,
) {
    for (entity, spawn) in tiles.iter() {
        match spawn {
            Spawn::Tile { set } => {
                commands
                    .entity(entity)
                    .insert(RigidBody::KinematicPositionBased)
                    .insert(PhysicsType::Obstacle.rapier())
                    .insert(Collider::cuboid(TILE_SIZE / 2., TILE_SIZE / 2.))
                    //
                    .insert(Depth::WallTile)
                    .insert(SimpleSprite {
                        images: assets.spritesets.get(&set).cloned().unwrap_or_default(),
                        frame: END_OF_TIMES,
                        size: Vec2::splat(TILE_SIZE),
                        ..default()
                    });
            }
        }
    }
}

#[derive(Default)]
struct SpriteSets {
    spritesets: HashMap<SpriteSet, Vec<Handle<Image>>>,
}

fn load_sprites(
    mut loading: ResMut<Loading>, server: Res<AssetServer>, mut assets: ResMut<SpriteSets>,
) {
    assets.spritesets.insert(
        SpriteSet::Player,
        loading.add_n(&server, "sprites/player/circ", 3, "png"),
    );
    assets.spritesets.insert(
        SpriteSet::Wall,
        loading.add_n(&server, "sprites/tiles/wall", 3, "png"),
    );
}
