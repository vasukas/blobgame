use bevy::{app::AppExit, log::LogSettings, prelude::*, window::PresentMode};
use bevy_egui::EguiPlugin;
use bevy_rapier2d::plugin::RapierPhysicsPlugin;
use control::menu::PlayNowHack;

// TODO: use leafwing-input-manager for ALL input except debug ones; also add keybinds

mod assets;
mod common;
mod control;
mod mechanics;
mod objects;
mod present;
mod settings;
mod utils;

fn main() {
    let mut app = App::new();

    // detect window size changes
    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugin(bevy_web_resizer::Plugin);
    }
    // exit app on Ctrl+Q
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_system_to_stage(
            CoreStage::Last,
            |keys: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>| {
                if keys.pressed(KeyCode::LControl) && keys.just_pressed(KeyCode::Q) {
                    exit.send_default();
                }
            },
        );
    }

    let mut args = std::env::args();
    args.next(); // skip program name
    if let Some(arg) = args.next() {
        match arg.as_str() {
            "play" => {
                app.insert_resource(PlayNowHack(true));
            }
            _ => panic!("Invalid command-line argument"),
        }
    }

    app.insert_resource(WindowDescriptor {
            title: "ScrapBot".to_string(),
            width: 1280.,
            height: 720.,

            #[cfg(target_os = "linux")] // avoid hanging on exit
            present_mode: PresentMode::Mailbox,

            fit_canvas_to_parent: true, // TODO: does it work? does it do that I think it does?
            ..default()
        })
    .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
    .insert_resource(LogSettings {
        filter: "wgpu=error,symphonia=error".into(),
        level: bevy::log::Level::INFO,
    })
    .insert_resource(settings::Settings::load().unwrap_or_default())
    .add_plugins(DefaultPlugins)
    .add_plugin(EguiPlugin)
    .add_plugin(bevy_prototype_lyon::plugin::ShapePlugin)
    .add_plugin(RapierPhysicsPlugin::<()>::pixels_per_meter(1.))
    .insert_resource({
        let mut config = bevy_rapier2d::plugin::RapierConfiguration::default();
        config.gravity = Vec2::ZERO; //-Vec2::Y * 10.;
        config
    })
    .add_plugin(control::ControlPlugin)
    .add_plugin(mechanics::MechanicsPlugin)
    .add_plugin(objects::ObjectsPlugin)
    .add_plugin(present::PresentationPlugin)
    .add_plugin(assets::MyAssetsPlugin)
    .run()
}
