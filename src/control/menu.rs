use super::{input::InputMap, time::TimeMode};
use crate::{common::*, objects::spawn::SpawnControl, present::camera::WindowInfo};
use bevy::app::AppExit;
use bevy_egui::EguiSettings;

#[derive(Default)]
pub struct PlayNowHack(pub bool);

pub fn is_exit_menu(keys: &Input<KeyCode>) -> bool {
    keys.any_just_pressed([KeyCode::Escape, KeyCode::M])
}

//

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .init_resource::<PlayNowHack>()
            .add_system(show_menu)
            .add_startup_system(setup)
            .add_startup_system(play_now_hack);
    }
}

#[derive(Default, PartialEq, Eq)]
enum MenuState {
    None,
    #[default]
    Root,
}

fn show_menu(
    mut ctx: ResMut<EguiContext>, mut state: ResMut<MenuState>, keys: Res<Input<KeyCode>>,
    mut exit_app: EventWriter<AppExit>, mut spawn: ResMut<SpawnControl>, window: Res<WindowInfo>,
    mut settings: ResMut<Settings>, input_map: Res<InputMap>, mut windows: ResMut<Windows>,
    mut time_ctl: ResMut<TimeMode>,
) {
    if is_exit_menu(&keys) && !time_ctl.craft_menu {
        match *state {
            MenuState::None => *state = MenuState::Root,
            MenuState::Root => {
                if spawn.is_game_running() {
                    *state = MenuState::None
                }
            }
        }
    }
    time_ctl.main_menu = *state != MenuState::None;

    match *state {
        MenuState::None => (),
        MenuState::Root => {
            let ingame = spawn.is_game_running();
            ctx.fill_screen(
                "menu::show_menu.bg",
                egui::Color32::from_black_alpha(255),
                egui::Order::Middle,
                window.size,
            );
            ctx.popup(
                "menu::show_menu",
                vec2(0., 0.),
                false,
                egui::Order::Foreground,
                |ui| {
                    egui::ScrollArea::both().show(ui, |ui| {
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
                                            *state = MenuState::None;
                                            spawn.despawn = Some(true)
                                        }
                                    }
                                    #[cfg(not(target_arch = "wasm32"))]
                                    if ui.button("Exit to desktop").clicked() {
                                        exit_app.send_default()
                                    }

                                    ui.label(""); // separator
                                    ui.heading("SETTINGS");
                                    let was_fullscreen = settings.fullscreen;
                                    settings.menu(ui);
                                    if was_fullscreen != settings.fullscreen {
                                        set_fullscreen(&mut windows, settings.fullscreen);
                                    }

                                    ui.label(""); // separator
                                    ui.label("Made for Bevy Jam #2");
                                    // TODO: add credits?
                                });
                            });

                            // root right pane
                            ui.vertical(|ui| {
                                ui.group(|ui| {
                                    ui.heading("CONTROLS");

                                    let mut help = vec![];
                                    for (i, (action, (key, ty))) in input_map.map.iter().enumerate()
                                    {
                                        help.push((
                                            action.description().to_string(),
                                            format!("{} ({})", key.to_string(), ty.description()),
                                        ));
                                        if i % 4 == 3 {
                                            help.push(("".to_string(), "".to_string()));
                                        }
                                    }
                                    help.push(("".to_string(), "".to_string()));
                                    help.push((
                                        "ESC or M".to_string(),
                                        "toggle this menu".to_string(),
                                    ));
                                    help.push(("Ctrl + Q".to_string(), "exit app".to_string()));

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
                },
            );
        }
    }
}

fn setup(mut egui: ResMut<EguiSettings>, settings: Res<Settings>, mut windows: ResMut<Windows>) {
    // TODO: correctly setup fonts instead of *this*
    egui.scale_factor *= 2.;
    set_fullscreen(&mut windows, settings.fullscreen);
}

fn play_now_hack(
    hack: Res<PlayNowHack>, mut spawn: ResMut<SpawnControl>, mut state: ResMut<MenuState>,
) {
    if hack.0 {
        spawn.despawn = Some(true);
        *state = MenuState::None;
    }
}

fn set_fullscreen(windows: &mut Windows, set: bool) {
    use bevy::window::WindowMode::*;
    let window = windows.primary_mut();
    window.set_mode(if set { BorderlessFullscreen } else { Windowed });
}
