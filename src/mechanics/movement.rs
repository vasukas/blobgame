use super::health::DieAfter;
use crate::{
    common::*,
    objects::{player::Player, spawn::TemporaryWall},
    present::hud_elements::WorldText,
};

#[derive(Component, Default)]
pub struct KinematicController {
    // config
    pub speed: f32,
    pub radius: f32,
    pub dash_distance: f32,
    pub dash_duration: Duration,

    // internal state
    pub dash: Option<(Vec2, Duration)>, // (dir, until)
}

/// Entity command
pub enum KinematicCommand {
    Move { dir: Vec2 },
    Dash { dir: Vec2 },
}

#[derive(Component, Default)]
pub struct DropSpread(Option<(Duration, Vec2)>);

#[derive(SystemLabel)]
pub struct MovementSystemLabel;

//

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<(Entity, KinematicCommand)>()
            .add_system(kinematic_controller.label(MovementSystemLabel))
            .add_system(drop_spread)
            .add_system(save_from_walls);
    }
}

// TODO: move this to player
fn save_from_walls(
    mut commands: Commands,
    mut players: Query<(Entity, &GlobalTransform, &mut Transform), With<KinematicController>>,
    walls: Query<(), With<TemporaryWall>>, phy: Res<RapierContext>,
) {
    for (entity, pos, mut transform) in players.iter_mut() {
        let in_wall = [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y]
            .into_iter()
            .all(|dir| {
                let radius = Player::RADIUS;
                let filter = QueryFilter::new()
                    .exclude_rigid_body(entity)
                    .groups(PhysicsType::MovementController.into());
                if let Some((entity, ..)) = phy.cast_ray(
                    pos.pos_2d() + dir * (radius * 0.9),
                    -dir,
                    radius * 0.2,
                    true,
                    filter,
                ) {
                    walls.contains(entity)
                } else {
                    false
                }
            });
        if in_wall {
            transform.set_2d(Vec2::ZERO);
            commands
                .spawn_bundle(SpatialBundle::default())
                .insert(WorldText {
                    text: vec![("SORRY".to_string(), Color::FUCHSIA)],
                    size: 2.,
                })
                .insert(DieAfter::new(Duration::from_millis(500)));
        }
    }
}

fn kinematic_controller(
    mut entities: Query<(
        Entity,
        &GlobalTransform,
        &mut Transform,
        &mut KinematicController,
    )>,
    time: Res<GameTime>, mut cmds: CmdReader<KinematicCommand>, phy: Res<RapierContext>,
) {
    // process commands
    cmds.iter_cmd_mut(
        &mut entities,
        |cmd, (entity, global_pos, mut transform, mut kinematic)| match *cmd {
            KinematicCommand::Move { dir } => {
                if let Some((dash, _)) = kinematic.dash.as_mut() {
                    *dash = dir;
                    return;
                }

                let ray_margin = -kinematic.speed * time.delta_seconds();

                let global_pos = global_pos.pos_2d();
                let filter = QueryFilter::new()
                    .exclude_rigid_body(entity)
                    .groups(PhysicsType::MovementController.into());

                let speed = kinematic.speed * time.delta_seconds();

                if phy
                    .cast_ray(
                        global_pos,
                        dir,
                        kinematic.radius + speed + ray_margin,
                        true,
                        filter,
                    )
                    .is_none()
                {
                    transform.add_2d(dir * speed);
                } else if phy
                    .cast_ray(
                        global_pos,
                        vec2(dir.x, 0.),
                        kinematic.radius + speed + ray_margin,
                        true,
                        filter,
                    )
                    .is_none()
                {
                    transform.add_2d(vec2(dir.x, 0.) * speed);
                } else if phy
                    .cast_ray(
                        global_pos,
                        vec2(0., dir.y),
                        kinematic.radius + speed + ray_margin,
                        true,
                        filter,
                    )
                    .is_none()
                {
                    transform.add_2d(vec2(0., dir.y) * speed);
                }
            }
            KinematicCommand::Dash { dir } => {
                kinematic.dash = Some((dir, time.now() + kinematic.dash_duration))
            }
        },
    );

    // process dash
    for (entity, global_pos, mut transform, mut kinematic) in entities.iter_mut() {
        if let Some((dir, until)) = kinematic.dash {
            if time.reached(until) {
                kinematic.dash = None
            } else {
                let global_pos = global_pos.pos_2d();
                let filter = QueryFilter::new()
                    .exclude_rigid_body(entity)
                    .groups(PhysicsType::MovementController.into());

                let speed = kinematic.dash_distance / kinematic.dash_duration.as_secs_f32();
                let offset = dir * speed * time.delta_seconds();
                if phy
                    .cast_ray(global_pos, offset, 1.1, true, filter)
                    .is_none()
                {
                    transform.add_2d(offset);
                }
            }
        }
    }
}

fn drop_spread(mut entities: Query<(&mut Transform, &mut DropSpread)>, time: Res<GameTime>) {
    let distance = 2.5; // approximate
    let duration = Duration::from_millis(1500);

    for (mut transform, mut spread) in entities.iter_mut() {
        let (start, dir) = spread.0.get_or_insert_with(|| {
            use rand::*;
            (
                time.now(),
                Vec2::Y.rotated(thread_rng().gen_range(0. ..TAU)),
            )
        });
        let t = time.t_passed(*start, duration);
        if t < 1. {
            let t = 1. - (t * TAU / 4.).sin();
            let speed = distance / duration.as_secs_f32() * time.delta_seconds() * t;
            transform.add_2d(*dir * speed)
        }
    }
}
