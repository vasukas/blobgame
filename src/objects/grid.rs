use super::{player::Player, spawn::WaveEvent};
use crate::{common::*, mechanics::health::Health};

#[derive(Component)]
pub struct GridBar {
    pub coord: f32, // [-1; 1]
    pub vertical: bool,
}

//

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GridPulse::idle())
            .add_system(draw_grid_pulse)
            .add_system(select_grid_pulse);
    }
}

struct GridPulse {
    period: Duration,
    color: Color,
    ty: PulseType,
}

enum PulseType {
    Uniform,
    Collapse,
    Expand,
}

impl GridPulse {
    fn idle() -> Self {
        GridPulse {
            period: Duration::from_millis(3000),
            color: Color::BLUE.with_a(0.7),
            ty: PulseType::Expand,
        }
    }
    fn alert() -> Self {
        GridPulse {
            period: Duration::from_millis(1500),
            color: Color::RED.with_a(0.6),
            ty: PulseType::Expand,
        }
    }
    fn waiting() -> Self {
        GridPulse {
            period: Duration::from_millis(1500),
            color: Color::CYAN.with_a(0.5),
            ty: PulseType::Collapse,
        }
    }
}

fn draw_grid_pulse(
    pulse: Res<GridPulse>, mut bars: Query<(&GridBar, &mut bevy_lyon::DrawMode)>,
    time: Res<GameTime>, mut wave: Local<f32>, mut color: Local<InterpolatedValue<Color>>,
) {
    color.length = Duration::from_millis(500);
    color.set_next(pulse.color);
    let color = color.update(time.now());

    *wave += time.delta_seconds() / pulse.period.as_secs_f32();

    for (bar, mut draw) in bars.iter_mut() {
        let t = match pulse.ty {
            PulseType::Uniform => wave.t_sin(),
            PulseType::Expand => {
                if bar.vertical {
                    (bar.coord.abs() - *wave).t_sin()
                } else {
                    0.
                }
            }
            PulseType::Collapse => {
                if bar.vertical {
                    (*wave + bar.coord.abs()).t_sin()
                } else {
                    0.
                }
            }
        };
        let mut color = color;
        color.set_a(pulse.color.a() * lerp(0.05, 0.15, t));
        *draw = bevy_lyon::DrawMode::Stroke(bevy_lyon::StrokeMode::new(color, 0.2));
    }
}

#[derive(Default)]
struct SelectState {
    wait_wave: bool,
}

fn select_grid_pulse(
    mut pulse: ResMut<GridPulse>, player: Query<&Health, With<Player>>,
    mut state: Local<SelectState>, mut wave_event: EventReader<WaveEvent>,
) {
    for ev in wave_event.iter() {
        match ev {
            WaveEvent::Started => state.wait_wave = false,
            WaveEvent::Ended => state.wait_wave = true,
        }
    }

    if state.wait_wave {
        *pulse = GridPulse::waiting()
    } else if let Ok(health) = player.get_single() {
        if health.value < health.max / 2. {
            *pulse = GridPulse::alert()
        } else {
            *pulse = GridPulse::idle()
        }
    }
}
