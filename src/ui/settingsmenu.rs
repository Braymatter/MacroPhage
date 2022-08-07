use bevy::window::WindowMode;
use bevy::{
    ecs::system::SystemParam,
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use bevy_egui::egui::style::Margin;
use bevy_egui::egui::{Color32, Frame, RichText, Stroke};
use bevy_egui::{
    egui,
    egui::{Align2, Grid, Window},
    EguiContext,
};
use directories::ProjectDirs;
use leafwing_input_manager::{prelude::*, user_input::InputKind};
use std::fs::{create_dir_all, File};
use std::io::Write;

use crate::{game::controller::PlayerAction, ui::UIState};
use crate::game::settings::{ActiveBinding, BindingConflict, ReadWriteGameSettings};
use crate::util::ui::set_ui_style;


use super::UIStateRes;

const UI_MARGIN: f32 = 10.0;

pub struct Images {
    save: Handle<Image>,
    cancel: Handle<Image>,
    edit_profile: Handle<Image>,
    save_id: egui::TextureId,
    cancel_id: egui::TextureId,
    edit_profile_id: egui::TextureId,
}

pub struct TinyImages {
    tiny_replace: Handle<Image>,
    tiny_cancel: Handle<Image>,
    tiny_replace_id: egui::TextureId,
    tiny_cancel_id: egui::TextureId,
}

const BTN_SIZE: (f32, f32) = (100., 40.);
const TINY_BTN_SIZE: (f32, f32) = (50., 20.);

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            save: asset_server.load("UI/save_and_exit.png"),
            cancel: asset_server.load("UI/cancel.png"),
            edit_profile: asset_server.load("UI/edit_profile.png"),
            save_id: egui::TextureId::default(),
            cancel_id: egui::TextureId::default(),
            edit_profile_id: egui::TextureId::default(),
        }
    }
}

impl FromWorld for TinyImages {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            tiny_replace: asset_server.load("UI/replace_smallest.png"),
            tiny_cancel: asset_server.load("UI/cancel_smallest.png"),
            tiny_replace_id: egui::TextureId::default(),
            tiny_cancel_id: egui::TextureId::default(),
        }
    }
}

//struct BindingConflict {
//action: PlayerAction,
//input_button: InputKind,
//}

#[allow(clippy::too_many_arguments)]
pub fn controls_window(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    windows: ResMut<Windows>,
    mut ui_state: ResMut<UIStateRes>,
    mut is_initialized: Local<bool>,
    mut images: Local<Images>,
    mut game_settings: ResMut<ReadWriteGameSettings>,
) {
    if !*is_initialized {
        *is_initialized = true;
        images.save_id = egui_context.add_image(images.save.clone_weak());
        images.cancel_id = egui_context.add_image(images.cancel.clone_weak());
        images.edit_profile_id = egui_context.add_image(images.edit_profile.clone_weak());
    }

    let main_window = windows.get_primary().unwrap();
    let window_width_margin = egui_context.ctx_mut().style().spacing.window_margin.left * 2.0;

    let controls = game_settings.pending_settings.inputs.clone();

    Window::new(RichText::new("Settings").color(Color32::WHITE).size(32.))
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .frame(Frame {
            fill: Color32::from_rgb(0, 38, 38),
            inner_margin: Margin::same(8.0),
            stroke: Stroke::new(0.6, Color32::from_rgb(50, 232, 214)),
            ..default()
        })
        .default_width(main_window.width() - UI_MARGIN * 2.0 - window_width_margin)
        .show(egui_context.ctx_mut(), |ui| {
            set_ui_style(ui);
            let btn_size = egui::vec2(BTN_SIZE.0, BTN_SIZE.1);

            const INPUT_VARIANTS: usize = 3;
            const COLUMNS_COUNT: usize = INPUT_VARIANTS + 1;

            Grid::new("Control grid")
                .num_columns(COLUMNS_COUNT)
                .striped(false)
                .min_col_width(ui.available_width() / COLUMNS_COUNT as f32 - window_width_margin)
                .show(ui, |ui| {
                    for action in PlayerAction::variants() {
                        ui.label(action.to_string());
                        let inputs = controls.get(action);
                        for index in 0..INPUT_VARIANTS {
                            let button_text = match inputs.get_at(index) {
                                Some(UserInput::Single(InputKind::GamepadButton(
                                    gamepad_button,
                                ))) => {
                                    format!("🎮 {:?}", gamepad_button)
                                }
                                Some(UserInput::Single(InputKind::Keyboard(keycode))) => {
                                    format!("🖮 {:?}", keycode)
                                }
                                Some(UserInput::Single(InputKind::Mouse(mouse_button))) => {
                                    format!("🖱 {:?}", mouse_button)
                                }
                                _ => "Empty".to_string(),
                            };
                            if ui.button(button_text).clicked() {
                                commands.insert_resource(ActiveBinding::new(action, index));
                            }
                        }
                        ui.end_row();
                    }
                });

            ui.checkbox(&mut game_settings.pending_settings.use_hardware_mouse, "Use hardware mouse");
            ui.checkbox(&mut game_settings.pending_settings.music_enabled, "Music enabled");

            egui::ComboBox::from_label("Display mode")
                .selected_text(format!("{:?}", game_settings.pending_settings.window_display_mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut game_settings.pending_settings.window_display_mode, WindowMode::Windowed, "Windowed");
                    ui.selectable_value(&mut game_settings.pending_settings.window_display_mode, WindowMode::BorderlessFullscreen, "Borderless Fullscreen");
                    ui.selectable_value(&mut game_settings.pending_settings.window_display_mode, WindowMode::SizedFullscreen, "Fullscreen (desktop)");
                    ui.selectable_value(&mut game_settings.pending_settings.window_display_mode, WindowMode::Fullscreen, "Fullscreen (max)");

                }
            );

            ui.horizontal(|ui| {
                ui.visuals_mut().widgets.inactive.expansion = -5.;  // bug with egui imagebutton padding
                let return_to_menu = ui.add(egui::ImageButton::new(images.save_id, btn_size)).clicked();
                let edit_profile = ui.add(egui::ImageButton::new(images.edit_profile_id, btn_size)).clicked();
                let cancel = ui.add(egui::ImageButton::new(images.cancel_id, btn_size)).clicked();
                ui.visuals_mut().widgets.inactive.expansion = 0.;   // end bug fix

                if return_to_menu {
                    // first save to struct
                    game_settings.actual_settings = game_settings.pending_settings.clone();

                    // now serialize to file
                    let json = serde_json::to_string(&game_settings.actual_settings);
                    if let Some(project_dirs) = ProjectDirs::from("", "", "macrophage") {
                        let path = project_dirs.config_dir();
                        create_dir_all(path).unwrap_or_else(|_| eprintln!("Error creating directories on config path {}.", path.display()));
                        let file = File::create(path.join("settings.json"));
                        match file {
                            Ok(mut file) => file.write_all(json.unwrap().as_bytes()).unwrap_or_else(|_| eprintln!("File write error on settings.json!")),
                            Err(_) => eprintln!("Error accessing settings.json file; it may be open in another program."),
                        };

                    }
                    ui_state.current_state = UIState::MainMenu;
                }

                if edit_profile {
                    // don't save settings if moving to profile screen
                    game_settings.pending_settings = game_settings.actual_settings.clone();
                    ui_state.current_state = UIState::Profile;
                }

                if cancel {
                    // reset settings
                    game_settings.pending_settings = game_settings.actual_settings.clone();
                    ui_state.current_state = UIState::MainMenu;
                }
            });

            ui.expand_to_include_rect(ui.available_rect_before_wrap());
        });
}

pub fn binding_window_system(
    mut commands: Commands,
    mut egui: ResMut<EguiContext>,
    mut input_events: InputEvents,
    active_binding: Option<ResMut<ActiveBinding>>,
    mut game_settings: ResMut<ReadWriteGameSettings>,
    mut images: Local<TinyImages>,
    mut is_initialized: Local<bool>,
) {
    if !*is_initialized {
        *is_initialized = true;
        images.tiny_cancel_id = egui.add_image(images.tiny_cancel.clone_weak());
        images.tiny_replace_id = egui.add_image(images.tiny_replace.clone_weak());
    }

    let mut active_binding = match active_binding {
        Some(active_binding) => active_binding,
        None => return,
    };

    Window::new(format!("Binding \"{}\"", active_binding.action))
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .frame(Frame {
            fill: Color32::from_rgb(0, 38, 38),
            inner_margin: Margin::same(8.0),
            stroke: Stroke::new(0.6, Color32::from_rgb(50, 232, 214)),
            ..default()
        })
        .show(egui.ctx_mut(), |ui| {
            set_ui_style(ui);
            let btn_size = egui::vec2(TINY_BTN_SIZE.0, TINY_BTN_SIZE.1);

            if let Some(conflict) = &active_binding.conflict {
                ui.label(format!(
                    "Input \"{}\" is already used by \"{}\"",
                    conflict.input_button, conflict.action
                ));
                ui.horizontal(|ui| {
                    ui.visuals_mut().widgets.inactive.expansion = -5.; // bug with egui imagebutton padding
                    let replace = ui.add(egui::ImageButton::new(images.tiny_replace_id, btn_size));
                    let cancel = ui.add(egui::ImageButton::new(images.tiny_cancel_id, btn_size));
                    ui.visuals_mut().widgets.inactive.expansion = 0.; // end bug fix

                    if replace.clicked() {
                        game_settings
                            .pending_settings
                            .inputs
                            .remove(conflict.action, conflict.input_button);
                        game_settings.pending_settings.inputs.insert_at(
                            conflict.input_button,
                            active_binding.action,
                            active_binding.index,
                        );
                        commands.remove_resource::<ActiveBinding>();
                    }
                    if cancel.clicked() {
                        commands.remove_resource::<ActiveBinding>();
                    }
                });
            } else {
                ui.label("Press any key now");
                if let Some(input_button) = input_events.input_button() {
                    let conflict_action = game_settings.pending_settings.inputs.iter().find_map(
                        |(inputs, action)| {
                            if action != active_binding.action
                                && inputs.contains(&input_button.into())
                            {
                                return Some(action);
                            }
                            None
                        },
                    );
                    if let Some(action) = conflict_action {
                        active_binding.conflict.replace(BindingConflict {
                            action,
                            input_button,
                        });
                    } else {
                        game_settings.pending_settings.inputs.insert_at(
                            input_button,
                            active_binding.action,
                            active_binding.index,
                        );
                        commands.remove_resource::<ActiveBinding>();
                    }
                }
            }
        });
}

#[derive(SystemParam)]
pub struct InputEvents<'w, 's> {
    keys: EventReader<'w, 's, KeyboardInput>,
    mouse_buttons: EventReader<'w, 's, MouseButtonInput>,
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

        None
    }
}
