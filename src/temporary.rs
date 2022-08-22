use crate::{
    common::*,
    present::camera::{WindowInfo, WorldCamera},
    utils::svg,
};

pub struct TemporaryPlugin;

impl Plugin for TemporaryPlugin {
    fn build(&self, app: &mut App) {
        app
            // fubar
            //.add_startup_system(test_svg_spawn)
            .add_system(tmp_keys)
            .add_system(debug_info);
    }
}

fn tmp_keys(
    mut camera: Query<(&mut WorldCamera, &mut Transform)>, keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if keys.just_pressed(KeyCode::Key1) {
        camera.single_mut().0.target_size *= 2.
    }
    if keys.just_pressed(KeyCode::Key2) {
        camera.single_mut().0.target_size /= 2.
    }

    let speed = 40. * time.delta_seconds();
    if keys.pressed(KeyCode::Left) {
        camera.single_mut().1.translation.x -= speed
    }
    if keys.pressed(KeyCode::Right) {
        camera.single_mut().1.translation.x += speed
    }
    if keys.pressed(KeyCode::Up) {
        camera.single_mut().1.translation.y += speed
    }
    if keys.pressed(KeyCode::Down) {
        camera.single_mut().1.translation.y -= speed
    }
}

fn debug_info(mut ctx: ResMut<EguiContext>, window: Res<WindowInfo>) {
    ctx.popup("temporary::debug_info", vec2(-1., -1.), true, |ui| {
        ui.label(format!("WINDOW SIZE {}", window.world_size()));
    });
}

#[allow(unused)]
fn test_svg_spawn(mut commands: Commands) {
    use bevy_lyon::*;

    let mut svg = svg::File::from_file("assets/levels/first.svg").unwrap();
    svg.fix();
    println!("SVG MINMAX {:?}", svg.minmax());

    let width = 0.1;
    for point in svg.points {
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shapes::Circle {
                    radius: point.radius,
                    center: point.pos,
                },
                DrawMode::Stroke(StrokeMode::new(Color::GREEN, width)),
                default(),
            ))
            .insert(Depth::TerrainOutline);
    }
    for line in svg.lines {
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shapes::Polygon {
                    points: line.pos.clone(),
                    closed: false,
                },
                DrawMode::Fill(FillMode::color(Color::GRAY)),
                default(),
            ))
            .insert(Depth::TerrainPolygon);

        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shapes::Polygon {
                    points: line.pos,
                    closed: false,
                },
                DrawMode::Stroke(StrokeMode::new(Color::WHITE, width)),
                default(),
            ))
            .insert(Depth::TerrainOutline);
    }
}
