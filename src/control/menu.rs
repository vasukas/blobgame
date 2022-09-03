use super::input::*;
use crate::{
    common::*,
    objects::{player::Player, spawn::SpawnControl},
    present::camera::WindowInfo,
};
use bevy::app::AppExit;
use bevy_egui::EguiSettings;
use leafwing_input_manager::plugin::ToggleActions;

// TODO: improve this
#[derive(Default)]
pub struct PlayNowHack(pub bool);

// TODO: REPLACE THIS
#[derive(Default, Debug)]
pub struct TimeMode {
    pub main_menu: bool,
    pub craft_menu: bool,
    pub player_alive: bool,
    pub overriden: Option<f32>,
}

impl TimeMode {
    pub fn stopped(&self) -> bool {
        self.main_menu || self.craft_menu || !self.player_alive
    }
}

//

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .init_resource::<PlayNowHack>()
            .init_resource::<TimeMode>()
            .add_system(show_menu)
            .add_system(mega_mode)
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
    mut ctx: ResMut<EguiContext>, mut state: ResMut<MenuState>,
    keys: Res<ActionState<ControlAction>>, mut exit_app: EventWriter<AppExit>,
    mut spawn: ResMut<SpawnControl>, window: Res<WindowInfo>, mut settings: ResMut<Settings>,
    mut windows: ResMut<Windows>, mut time_mode: ResMut<TimeMode>,
) {
    if keys.just_pressed(ControlAction::ExitMenu) && !time_mode.craft_menu {
        match *state {
            MenuState::None => *state = MenuState::Root,
            MenuState::Root => {
                if spawn.is_game_running() {
                    *state = MenuState::None
                }
            }
        }
    }
    time_mode.main_menu = *state != MenuState::None;

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
                                    ui.heading("SCRAPBOT");
                                    ui.label(""); // separator

                                    if ingame {
                                        if ui.button("Continue").clicked() {
                                            *state = MenuState::None
                                        }
                                        if ui.button("Restart wave").clicked() {
                                            *state = MenuState::None;
                                            spawn.despawn = Some(true);
                                        }
                                        if ui.button("Exit to main menu").clicked() {
                                            spawn.despawn = Some(false)
                                        }
                                    } else {
                                        if ui.button("Play (with tutorial)").clicked() {
                                            *state = MenuState::None;
                                            spawn.despawn = Some(true);
                                            spawn.tutorial = Some(0);
                                        }
                                        if ui.button("Play (skip tutorial)").clicked() {
                                            *state = MenuState::None;
                                            spawn.despawn = Some(true);
                                            spawn.tutorial = None;
                                        }
                                    }
                                    #[cfg(not(target_arch = "wasm32"))]
                                    if ui.button("Exit to desktop").clicked() {
                                        exit_app.send_default()
                                    }

                                    ui.label(""); // separator
                                    ui.heading("SETTINGS");
                                    let was_fullscreen = settings.fullscreen;
                                    if settings.menu(ui) {
                                        settings.save()
                                    }
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

                                    // TODO: apply settings
                                    if settings.input.menu(ui) {
                                        settings.save()
                                    }
                                });
                            });
                        });
                    });
                },
            );
        }
    }
}

fn mega_mode(
    mut mode: ResMut<TimeMode>, mut time: ResMut<GameTime>,
    mut player: ResMut<ToggleActions<PlayerAction>>, mut craft: ResMut<ToggleActions<CraftAction>>,
    mut control: ResMut<ToggleActions<ControlAction>>, actual_player: Query<(), With<Player>>,
) {
    mode.player_alive = !actual_player.is_empty();

    let lock_active = mode.main_menu || mode.craft_menu;
    let lock_allow_craft = mode.craft_menu;
    time.scale = if mode.stopped() { 0. } else { mode.overriden.unwrap_or(1.) };

    player.enabled = !lock_active;
    craft.enabled = !lock_active || lock_allow_craft;
    control.enabled = !lock_active;
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
