use crate::common::*;

#[derive(Component, Default)]
pub struct KinematicController {
    // config
    pub speed: f32,
    pub radius: f32,

    // internal state
    pub state: (),
}

/// Entity command
pub enum KinematicCommand {
    Move { dir: Vec2 },
}

#[derive(SystemLabel)]
pub struct MovementSystemLabel;

//

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<(Entity, KinematicCommand)>()
            .add_system(kinematic_controller.label(MovementSystemLabel));
    }
}

fn kinematic_controller(
    mut entities: Query<(
        Entity,
        &GlobalTransform,
        &mut Transform,
        &KinematicController,
        &CollisionGroups,
    )>,
    time: Res<GameTime>, mut cmds: CmdReader<KinematicCommand>, phy: Res<RapierContext>,
) {
    cmds.iter_cmd_mut(
        &mut entities,
        |cmd, (entity, global_pos, mut transform, kinematic, groups)| {
            match *cmd {
                // horizontal movement
                KinematicCommand::Move { dir } => {
                    let ray_margin = -kinematic.speed * time.delta_seconds();

                    let global_pos = global_pos.pos_2d();
                    let filter = QueryFilter::new()
                        .exclude_rigid_body(entity)
                        .groups((*groups).into());

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
                    }
                }
            }
        },
    );
}
