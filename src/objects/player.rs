use crate::{
    common::*,
    control::input::InputAction,
    mechanics::{damage::Team, movement::*},
    present::{simple_sprite::SimpleSprite, sound::AudioListener},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, spawn_player.exclusive_system())
            .add_system(controls.before(MovementSystemLabel));
    }
}

#[derive(Component, Default)]
pub struct Player {
    //
}

fn spawn_player(
    mut commands: Commands, player: Query<Entity, Added<Player>>, assets: Res<MyAssets>,
) {
    for entity in player.iter() {
        let radius = 0.5;

        commands
            .entity(entity)
            .insert(KinematicController {
                speed: 8.,
                radius,
                ..default()
            })
            .insert(RigidBody::KinematicPositionBased)
            .insert(Collider::ball(radius * 0.9))
            .insert(PhysicsType::Solid.rapier())
            //
            .insert(Depth::Player)
            .insert(SimpleSprite {
                images: assets.player.clone(),
                frame_duration: Duration::from_millis(250),
                size: Vec2::splat(radius * 2.),
                ..default()
            })
            .with_children(|parent| {
                use bevy_lyon::*;
                parent.spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Circle {
                        radius: radius * 0.9,
                        center: Vec2::ZERO,
                    },
                    DrawMode::Fill(FillMode::color(Color::CYAN * 0.5)),
                    default(),
                ));
            })
            .insert(AudioListener)
            .insert(Team::Player);
    }
}

fn controls(
    mut player: Query<(Entity, &mut Player)>, mut input: EventReader<InputAction>,
    mut kinematic: CmdWriter<KinematicCommand>,
) {
    let (entity, mut _player) = match player.get_single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };

    let mut mov = Vec2::ZERO;
    for action in input.iter() {
        match action {
            InputAction::MoveLeft => mov.x -= 1.,
            InputAction::MoveRight => mov.x += 1.,
            InputAction::MoveUp => mov.y += 1.,
            InputAction::MoveDown => mov.y -= 1.,
        }
    }
    if let Some(dir) = mov.try_normalize() {
        kinematic.send((entity, KinematicCommand::Move { dir }))
    }
}
