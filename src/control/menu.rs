use super::level::LevelCommand;
use crate::{common::*, present::camera::WindowInfo};
use bevy::app::AppExit;
use bevy_egui::EguiSettings;

/// This is a hack. It contains a hack. Dear God.
pub struct StartupMenuHack {
    pub play_level: Option<String>,
}

const FIRST_LEVEL: &str = "levels/first.svg";

/// In-game UI must be drawn before this!
#[derive(SystemLabel)]
pub struct UiMenuSystem;

//

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MainState::MainMenu)
            .add_system(show_menu.label(UiMenuSystem))
            .add_system_to_stage(CoreStage::First, set_time)
            .add_startup_system(play_level)
            .add_startup_system(setup_egui_scale);
    }
}

// TODO: correctly setup fonts instead of *this*
fn setup_egui_scale(mut egui: ResMut<EguiSettings>) {
    egui.scale_factor *= 2.
}

enum MainState {
    MainMenu,
    Game,
    InGameMenu,
}

fn show_menu(
    mut ctx: ResMut<EguiContext>, mut state: ResMut<MainState>, keys: Res<Input<KeyCode>>,
    mut exit_app: EventWriter<AppExit>, mut level_cmd: EventWriter<LevelCommand>,
    server: Res<AssetServer>, window: Res<WindowInfo>, mut settings: ResMut<Settings>,
) {
    if keys.any_just_pressed([KeyCode::Escape, KeyCode::F10]) {
        *state = match *state {
            MainState::MainMenu => MainState::MainMenu, // no-op
            MainState::Game => MainState::InGameMenu,
            MainState::InGameMenu => MainState::Game,
        };
    }
    match *state {
        MainState::Game => (),
        MainState::MainMenu | MainState::InGameMenu => {
            let ingame = match *state {
                MainState::MainMenu => false,
                MainState::Game | MainState::InGameMenu => true,
            };
            if ingame {
                ctx.fill_screen(
                    "menu::show_menu.bg",
                    egui::Color32::from_black_alpha(128),
                    window.size,
                );
            }
            ctx.popup("menu::show_menu", vec2(0., 0.), false, |ui| {
                ui.heading("BLOBFIGHT");
                ui.label(""); // separator

                if ingame {
                    if ui.button("Continue").clicked() {
                        *state = MainState::Game
                    }
                    if ui.button("Exit to main menu").clicked() {
                        *state = MainState::MainMenu;
                        level_cmd.send(LevelCommand::Unload)
                    }
                } else {
                    if ui.button("Play").clicked() {
                        *state = MainState::Game;
                        level_cmd.send(LevelCommand::Load(server.load(FIRST_LEVEL)))
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                if ui.button("Exit to desktop").clicked() {
                    exit_app.send_default()
                }

                ui.label(""); // separator
                settings.menu(ui);

                ui.label(""); // separator
                ui.label("Made by vasukas with Bevy Engine");
                // TODO: don't forget to add other stuff here
            });
        }
    }
}

fn set_time(mut time: ResMut<GameTime>, state: Res<MainState>) {
    time.scale = match *state {
        MainState::Game => 1.,
        MainState::MainMenu | MainState::InGameMenu => 0.,
    }
}

fn play_level(
    hack: Res<StartupMenuHack>, mut state: ResMut<MainState>,
    mut level_cmd: EventWriter<LevelCommand>, server: Res<AssetServer>,
) {
    if let Some(level) = hack.play_level.as_ref() {
        level_cmd.send(LevelCommand::Load(
            server.load(&format!("levels/{}.svg", level)),
        ));
        *state = MainState::Game;
    }
}
