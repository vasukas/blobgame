use super::input::*;
use crate::{
    common::*,
    objects::{player::Player, spawn::SpawnControl},
    present::camera::WindowInfo,
};
use bevy::{
    app::AppExit,
    ecs::system::SystemParam,
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
};
use bevy_egui::EguiSettings;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

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
    mut windows: ResMut<Windows>, mut time_mode: ResMut<TimeMode>, mut input_events: InputEvents,
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

    if *state != MenuState::None {
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
                    match *state {
                        MenuState::None => unimplemented!(),

                        MenuState::Root => {
                            ui.horizontal(|ui| {
                                // left pane - general
                                ui.vertical(|ui| {
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

                                    // settings

                                    let mut changed = false;

                                    ui.horizontal(|ui| {
                                        ui.label("Master volume");
                                        changed |= ui
                                            .add(egui::Slider::new(
                                                &mut settings.master_volume,
                                                0. ..=1.,
                                            ))
                                            .changed();
                                    });

                                    if ui
                                        .checkbox(&mut settings.fullscreen, "Fullscreen")
                                        .changed()
                                    {
                                        changed = true;
                                        set_fullscreen(&mut windows, settings.fullscreen);
                                    }
                                    #[cfg(target_arch = "wasm32")]
                                    ui.label(
                                        "If it doesn't change, click anywhere again or something",
                                    );

                                    ui.label(
                                        "Difficulty: Hard. Other levels will be implemented later.",
                                    );
                                    // let (alt, text) = match settings.difficulty {
                                    //     Difficulty::Easy => (Difficulty::Hard, "Difficulty: Easy"),
                                    //     Difficulty::Hard => (Difficulty::Easy, "Difficulty: Hard"),
                                    // };
                                    // changed |= {
                                    //     let clicked = ui.button(text).clicked();
                                    //     if clicked {
                                    //         settings.difficulty = alt
                                    //     }
                                    //     clicked
                                    // };
                                    // ui.label("Changes to difficulty will be applied after respawn");

                                    if changed {
                                        settings.save()
                                    }

                                    // settings ended

                                    ui.label(""); // separator
                                    ui.label("Made with Bevy engine");
                                    // TODO: add credits?
                                });

                                // right pane - controls
                                ui.vertical(|ui| {
                                    ui.heading("CONTROLS");
                                    ui.label(""); // separator

                                    // Based on https://github.com/Leafwing-Studios/leafwing-input-manager/blob/main/examples/binding_menu.rs

                                    egui::Grid::new("settings controls").show(ui, |ui| {
                                        for action in PlayerAction::variants() {
                                            ui.label(action.description());
                                            for v in settings.input.player.get(action).iter() {
                                                let text = match v {
                                                    UserInput::Single(InputKind::Keyboard(v)) => {
                                                        format!("{:?} key", v)
                                                    }
                                                    UserInput::Single(InputKind::Mouse(v)) => {
                                                        format!("{:?} button", v)
                                                    }
                                                    UserInput::Single(InputKind::MouseWheel(v)) => {
                                                        format!("{:?} wheel", v)
                                                    }
                                                    UserInput::Single(
                                                        InputKind::GamepadButton(v),
                                                    ) => {
                                                        format!("{:?}", v)
                                                    }
                                                    UserInput::Single(_) => "<SINGLE>".to_string(),
                                                    UserInput::Chord(_) => "<CHORD>".to_string(),
                                                    UserInput::VirtualDPad(_) => {
                                                        "<VDPAD>".to_string()
                                                    }
                                                };
                                                ui.label(text);
                                                // TODO: implement actual binding
                                            }
                                            ui.end_row()
                                        }
                                    });
                                });
                            })
                        }
                    }
                });
            },
        );
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

/// Helper for collecting input
/// Copied from https://github.com/Leafwing-Studios/leafwing-input-manager/blob/main/examples/binding_menu.rs
#[derive(SystemParam)]
struct InputEvents<'w, 's> {
    keys: EventReader<'w, 's, KeyboardInput>,
    mouse_buttons: EventReader<'w, 's, MouseButtonInput>,
    gamepad_events: EventReader<'w, 's, GamepadEvent>,
}

impl InputEvents<'_, '_> {
    fn input_button(&mut self) -> Option<InputKind> {
        if let Some(keyboard_input) = self.keys.iter().next() {
            if keyboard_input.state == ButtonState::Released {
                if let Some(key_code) = keyboard_input.key_code {
                    return Some(key_code.into());
                }
            }
        }
        if let Some(mouse_input) = self.mouse_buttons.iter().next() {
            if mouse_input.state == ButtonState::Released {
                return Some(mouse_input.button.into());
            }
        }
        if let Some(GamepadEvent {
            gamepad: _,
            event_type,
        }) = self.gamepad_events.iter().next()
        {
            if let GamepadEventType::ButtonChanged(button, strength) = event_type.to_owned() {
                if strength <= 0.5 {
                    return Some(button.into());
                }
            }
        }
        None
    }
}
