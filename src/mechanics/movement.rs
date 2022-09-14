use crate::common::*;

#[derive(Component, Default)]
pub struct KinematicController {
    // config
    pub speed: f32,
    pub radius: f32,
    pub dash_distance: f32,
    pub dash_duration: Duration,

    // internal state
    pub dash: Option<(Vec2, Duration)>, // (dir, until)
    pub mov: Option<Vec2>,
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
            .add_system(drop_spread);
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
    let toi_margin = 0.05;

    // process commands
    cmds.iter_entities(&mut entities, |cmd, (.., mut kinematic)| match *cmd {
        KinematicCommand::Move { dir } => match kinematic.dash.as_mut() {
            Some((dash, ..)) => *dash = dir,
            None => {
                let speed = kinematic.speed;
                *kinematic.mov.get_or_insert(default()) += dir * speed
            }
        },
        KinematicCommand::Dash { dir } => {
            kinematic.dash = Some((dir, time.now() + kinematic.dash_duration))
        }
    });

    // process movement
    for (entity, global_pos, mut transform, mut kinematic) in entities.iter_mut() {
        let kinematic = &mut *kinematic;

        if let Some((dir, until)) = kinematic.dash {
            if time.reached(until) {
                kinematic.dash = None
            } else {
                let speed = kinematic.dash_distance / kinematic.dash_duration.as_secs_f32();
                kinematic.mov = Some(dir * speed)
            }
        }

        let mov = kinematic.mov.take().unwrap_or_default() * time.delta_seconds();
        transform.add_2d(mov);

        let global_pos = global_pos.pos_2d() + mov;
        let filter = QueryFilter::new()
            .exclude_rigid_body(entity)
            .groups(PhysicsType::MovementController.into());

        for dir in [-Vec2::Y, Vec2::Y, -Vec2::X, Vec2::X] {
            let length = kinematic.radius;
            let toi = [
                Vec2::ZERO,
                // dir.clockwise90() * length,
                // dir.clockwise90() * -length,
            ]
            .into_iter()
            .map(|offset| {
                phy.cast_ray(global_pos + offset, dir, length + toi_margin, false, filter)
                    .map(|(_, toi)| toi)
            })
            .filter_map(|x| x)
            .reduce(f32::min);

            if let Some(toi) = toi {
                let fix = length - toi;
                if fix > 0. {
                    transform.add_2d(dir * -fix)
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
