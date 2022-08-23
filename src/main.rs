//#![allow(unstable_name_collisions)] // I swear I don't do anything too weird with this

use bevy::{app::AppExit, prelude::*, window::PresentMode};
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy_rapier2d::plugin::RapierPhysicsPlugin;
use control::menu::StartupMenuHack;

// TODO: use leafwing-input-manager for ALL input except debug ones; also add keybinds

mod assets;
mod common;
mod control;
mod mechanics;
mod objects;
mod present;
mod settings;
mod temporary;
mod utils;

fn main() {
    let mut args = std::env::args();
    args.next(); // skip program name
    let play_level = args.next();

    App::new()
        .insert_resource(WindowDescriptor {
            title: "BlobFight".to_string(),
            width: 1280.,
            height: 720.,

            #[cfg(target_os = "linux")] // avoid hanging on exit
            present_mode: PresentMode::Mailbox,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(StartupMenuHack { play_level })
        .insert_resource(settings::Settings::load().unwrap_or_default())
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(RapierPhysicsPlugin::<()>::pixels_per_meter(1.))
        .insert_resource({
            let mut config = bevy_rapier2d::plugin::RapierConfiguration::default();
            config.gravity = Vec2::ZERO;
            config
        })
        .add_system_to_stage(CoreStage::Last, exit_on_esc_system)
        .add_startup_system(setup)
        .add_plugin(control::ControlPlugin)
        .add_plugin(mechanics::MechanicsPlugin)
        .add_plugin(objects::ObjectsPlugin)
        .add_plugin(present::PresentationPlugin)
        .add_plugin(temporary::TemporaryPlugin)
        .add_plugin(assets::MyAssetsPlugin)
        .run()
}

fn setup(mut windows: ResMut<Windows>, server: Res<AssetServer>) {
    windows.primary_mut().set_maximized(true);
    server.watch_for_changes().unwrap();
}

fn exit_on_esc_system(keys: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.pressed(KeyCode::LControl) && keys.just_pressed(KeyCode::Q) {
        exit.send_default();
    }
}
