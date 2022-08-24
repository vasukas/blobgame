use crate::common::*;

#[derive(Component, Default, Clone, Copy)]
pub struct Light {
    pub radius: f32,
    pub color: Color,
}

/// Requires `Light` component to be present (just default-init it)
#[derive(Component, Default)]
pub struct LightPulse {
    pub period: Duration,
    pub source: Light,

    // internal
    pub state: Option<(Duration, Duration)>,
}

//

pub struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            spawn_light.exclusive_system().at_end(),
        )
        .add_system(update_light)
        .add_system(light_pulse.before(update_light));
    }
}

#[derive(Component)]
struct LightChild(Entity);

fn spawn_light(
    mut commands: Commands, lights: Query<(Entity, &Light), Added<Light>>, assets: Res<MyAssets>,
) {
    for (entity, light) in lights.iter() {
        let mut child = BadEntityHack::default();
        commands
            .entity(entity)
            .with_children(|parent| {
                child.set(
                    parent
                        .spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                color: light.color,
                                custom_size: Some(Vec2::splat(light.radius)),
                                ..default()
                            },
                            texture: assets.glow.clone(),
                            ..default()
                        })
                        .insert(Depth::Light)
                        .id(),
                );
            })
            .insert(LightChild(child.get()));
    }
}

fn update_light(
    lights: Query<(&Light, &LightChild), Changed<Light>>, mut sprites: Query<&mut Sprite>,
) {
    for (light, child) in lights.iter() {
        if let Ok(mut sprite) = sprites.get_mut(child.0) {
            sprite.color = light.color;
            sprite.custom_size = Some(Vec2::splat(light.radius));
        }
    }
}

fn light_pulse(mut entities: Query<(&mut Light, &mut LightPulse)>, time: Res<GameTime>) {
    for (mut light, mut pulse) in entities.iter_mut() {
        if pulse
            .state
            .as_ref()
            .map(|state| time.reached(state.0 + state.1))
            .unwrap_or(true)
        {
            use rand::*;
            let period = pulse.period.mul_f32(thread_rng().gen_range(0.8..1.2));
            pulse.state = Some((time.now(), period));
        }
        let (start, period) = pulse.state.unwrap();
        *light = pulse.source;
        light.radius *= lerp(0.9, 1.1, time.t_passed(start, period).t_sin());
    }
}
