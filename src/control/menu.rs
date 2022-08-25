use super::input::{InputLock, InputMap};
use crate::{common::*, objects::spawn::SpawnControl, present::camera::WindowInfo};
use bevy::app::AppExit;
use bevy_egui::EguiSettings;

#[derive(Default)]
pub struct PlayNowHack(pub bool);

/// In-game UI must be drawn before this!
#[derive(SystemLabel)]
pub struct UiMenuSystem;

//

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .init_resource::<PlayNowHack>()
            .add_system(show_menu.label(UiMenuSystem))
            .add_system_to_stage(CoreStage::First, set_time)
            .add_startup_system(setup_egui_scale)
            .add_startup_system(play_now_hack);
    }
}

#[derive(Default)]
enum MenuState {
    None,
    #[default]
    Root,
}

fn show_menu(
    mut ctx: ResMut<EguiContext>, mut state: ResMut<MenuState>, keys: Res<Input<KeyCode>>,
    mut exit_app: EventWriter<AppExit>, mut spawn: ResMut<SpawnControl>, window: Res<WindowInfo>,
    mut settings: ResMut<Settings>, mut input_lock: ResMut<InputLock>, input_map: Res<InputMap>,
) {
    if keys.any_just_pressed([KeyCode::Escape, KeyCode::F10]) {
        match *state {
            MenuState::None => *state = MenuState::Root,
            MenuState::Root => {
                if spawn.spawned {
                    *state = MenuState::None
                }
            }
        }
    }
    input_lock.active = match *state {
        MenuState::None => false,
        _ => true,
    };

    match *state {
        MenuState::None => (),
        MenuState::Root => {
            let ingame = spawn.spawned;
            ctx.fill_screen(
                "menu::show_menu.bg",
                egui::Color32::from_black_alpha(255),
                window.size,
            );
            ctx.popup("menu::show_menu", vec2(0., 0.), false, |ui| {
                ui.horizontal(|ui| {
                    // root left pane
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.heading("BLOBFIGHT");
                            ui.label(""); // separator

                            if ingame {
                                if ui.button("Continue").clicked() {
                                    *state = MenuState::None
                                }
                                if ui.button("Exit to main menu").clicked() {
                                    spawn.despawn = Some(false)
                                }
                            } else {
                                if ui.button("Play").clicked() {
                                    spawn.despawn = Some(true)
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
                    });
                    // root right pane
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.heading("HELP");

                            let mut help = vec![
                                ("ESC or F10".to_string(), "toggle this menu".to_string()),
                                ("Ctrl + Q".to_string(), "exit app".to_string()),
                                ("".to_string(), "".to_string()),
                            ];
                            for (action, (key, ty)) in input_map.map.iter() {
                                help.push((
                                    action.description().to_string(),
                                    format!("{} ({})", key.to_string(), ty.description()),
                                ));
                            }

                            // poor man's table
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    for v in &help {
                                        ui.label(&v.0);
                                    }
                                });
                                ui.vertical(|ui| {
                                    for v in &help {
                                        ui.label(&v.1);
                                    }
                                });
                            });
                        });
                    });
                });
            });
        }
    }
}

fn set_time(mut time: ResMut<GameTime>, state: Res<MenuState>) {
    time.scale = match *state {
        MenuState::None => 1.,
        _ => 0.,
    }
}

// TODO: correctly setup fonts instead of *this*
fn setup_egui_scale(mut egui: ResMut<EguiSettings>) {
    egui.scale_factor *= 2.
}

fn play_now_hack(
    hack: Res<PlayNowHack>, mut spawn: ResMut<SpawnControl>, mut state: ResMut<MenuState>,
) {
    if hack.0 {
        spawn.despawn = Some(true);
        *state = MenuState::None;
    }
}
