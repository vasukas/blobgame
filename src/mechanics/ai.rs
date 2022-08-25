use crate::{
    common::*,
    objects::{
        player::Player,
        weapon::{Weapon, WeaponSystemLabel},
    },
};

/// What should be targeted
#[derive(Component)]
pub enum Target {
    Player,
}

/// Check line of sight to the target.
/// Requires Target
#[derive(Component, Default)]
pub struct LosCheck {
    // config
    pub dir: Vec2,
    pub distance: f32, // non-squared

    // state
    pub visible: bool,
}

/// Rotate so it always tries to face the target.
/// Requires LosCheck
#[derive(Component, Default)]
pub struct FaceTarget {
    // config
    pub rotation_speed: f32,
    pub disabled: bool,

    // state
    pub angle: f32,
}

/// Requires LosCheck
#[derive(Component, Default)]
pub struct AttackPattern {
    // config
    pub stages: Vec<(Duration, AttackStage)>,

    // state
    pub stage: usize,
    pub start: Option<Duration>,
}

impl AttackPattern {
    pub fn stage(mut self, repeat: usize, duration: Duration, stage: AttackStage) -> Self {
        for _ in 0..repeat {
            self.stages.push((duration, stage))
        }
        self
    }
}

#[derive(Clone, Copy)]
pub enum AttackStage {
    Wait,
    Shoot(Weapon),
}

//

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(new.exclusive_system().at_start())
            .add_system(update_target)
            .add_system(los_check.after(update_target))
            .add_system(face_target.after(los_check))
            .add_system(attack_pattern.after(face_target).before(WeaponSystemLabel));
    }
}

fn new(
    mut commands: Commands, targets: Query<Entity, Added<Target>>,
    mut faces: Query<(&GlobalTransform, &mut FaceTarget), Added<FaceTarget>>,
) {
    for entity in targets.iter() {
        commands.entity(entity).insert(TargetState::default());
    }
    for (transform, mut face) in faces.iter_mut() {
        face.angle = transform.pos_2d().angle()
    }
}

#[derive(Component, Default)]
struct TargetState(Option<Entity>);

fn update_target(
    mut state: Query<(&Target, &mut TargetState)>, player: Query<Entity, With<Player>>,
) {
    let player = player.get_single().ok();
    for (target, mut state) in state.iter_mut() {
        match *target {
            Target::Player => state.0 = player,
        }
    }
}

fn los_check(
    mut entities: Query<(&GlobalTransform, &TargetState, &mut LosCheck)>,
    targets: Query<&GlobalTransform>, phy: Res<RapierContext>,
) {
    for (origin, target, mut los) in entities.iter_mut() {
        los.visible = if let Some((target_pos, target)) =
            target.0.and_then(|e| targets.get(e).ok()).zip(target.0)
        {
            los.dir = target_pos.pos_2d() - origin.pos_2d();
            los.distance = los.dir.length();

            if let Some((hit, _)) = phy.cast_ray(
                origin.pos_2d(),
                los.dir,
                1.,
                false,
                QueryFilter::new().groups(PhysicsType::Solid.into()),
            ) {
                hit == target
            } else {
                // that... shouldn't happen
                true
            }
        } else {
            false
        };
    }
}

fn face_target(
    mut entities: Query<(&mut Transform, &mut FaceTarget, &LosCheck)>, time: Res<GameTime>,
) {
    for (mut transform, mut target, los) in entities.iter_mut() {
        if target.disabled {
            continue;
        }
        let speed = target.rotation_speed * time.delta_seconds();
        let delta = angle_delta(los.dir.angle(), target.angle).clamp(-speed, speed);
        target.angle += delta;
        transform.set_angle_2d(target.angle);
    }
}

fn attack_pattern(
    mut entities: Query<(Entity, &mut AttackPattern)>, time: Res<GameTime>,
    mut weapon_commands: CmdWriter<Weapon>,
) {
    for (entity, mut pattern) in entities.iter_mut() {
        if pattern.stages.is_empty() {
            continue;
        }

        let passed = time.passed(*pattern.start.get_or_insert(time.now()));
        let duration = pattern
            .stages
            .get(pattern.stage)
            .map(|v| v.0)
            .unwrap_or_default();
        if passed >= duration {
            pattern.stage += 1;
            pattern.start = None;
            if pattern.stage >= pattern.stages.len() {
                pattern.stage = 0;
            }

            match pattern.stages.get(pattern.stage).unwrap().1 {
                AttackStage::Wait => (),
                AttackStage::Shoot(weapon) => weapon_commands.send((entity, weapon)),
            }
        }
    }
}
