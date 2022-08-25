use super::spawn::SpawnControl;
use crate::{
    common::*,
    control::{
        input::{InputAction, InputMap},
        menu::UiMenuSystem,
    },
    mechanics::{damage::Team, health::Health, movement::*},
    present::{camera::WindowInfo, simple_sprite::SimpleSprite, sound::AudioListener},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PreUpdate, spawn_player.exclusive_system())
            .add_system(controls.before(MovementSystemLabel))
            .add_system(respawn.before(UiMenuSystem));
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
            //
            .insert(Team::Player)
            // TODO: increase health back
            .insert(Health::new(3.).armor());
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
            _ => (),
        }
    }
    if let Some(dir) = mov.try_normalize() {
        kinematic.send((entity, KinematicCommand::Move { dir }))
    }
}

#[derive(Default)]
struct RespawnMenu {
    death_start: Option<Duration>,
}

fn respawn(
    mut ctx: ResMut<EguiContext>, mut data: Local<RespawnMenu>, player: Query<With<Player>>,
    time: Res<Time>, mut spawn: ResMut<SpawnControl>, mut input: EventReader<InputAction>,
    input_map: Res<InputMap>, window: Res<WindowInfo>,
) {
    if player.is_empty() {
        let passed =
            time.time_since_startup() - *data.death_start.get_or_insert(time.time_since_startup());
        let t = passed.as_secs_f32() / Duration::from_secs(2).as_secs_f32();
        ctx.fill_screen(
            "player::respawn.bg",
            egui::Color32::from_black_alpha((t * 255.).min(255.) as u8),
            window.size,
        );
        ctx.popup("player::respawn", Vec2::ZERO, false, |ui| {
            ui.heading("~= YOU DIED =~");

            ui.horizontal(|ui| {
                // TODO: where should be a better way to make multicolored text
                ui.label("Press [");
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.label(input_map.map[InputAction::Respawn].0.to_string());
                ui.visuals_mut().override_text_color = None;
                ui.label("] to restart from checkpoint");
            });
        });
        for action in input.iter() {
            match action {
                InputAction::Respawn => spawn.despawn = Some(true),
                _ => (),
            }
        }
    } else {
        data.death_start = None;
    }
}
