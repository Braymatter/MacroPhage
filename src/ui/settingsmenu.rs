use std::fs::{create_dir_all, File};
use std::io::Write;
use bevy::{
    ecs::system::SystemParam,
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ElementState},
    prelude::*,
};
use bevy::window::WindowMode;
use bevy_egui::{egui::{Align2, Grid, Window}, egui, EguiContext};
use bevy_egui::egui::{Color32, Frame};
use directories::ProjectDirs;
use leafwing_input_manager::{prelude::*, user_input::InputButton};

use crate::{game::controller::PlayerAction, ui::UIState};
use crate::ui::{GameSettings, ReadWriteGameSettings};
use crate::util::ui::set_ui_style;

use super::UIStateRes;

const UI_MARGIN: f32 = 10.0;

//Stolen from: https://github.com/Leafwing-Studios/leafwing-input-manager/blob/main/examples/binding_menu.rs#L2
pub struct ActiveBinding {
    action: PlayerAction,
    index: usize,
    conflict: Option<BindingConflict>,
}

impl ActiveBinding {
    fn new(action: PlayerAction, index: usize) -> Self {
        Self {
            action,
            index,
            conflict: None,
        }
    }
}

struct BindingConflict {
    action: PlayerAction,
    input_button: InputButton,
}

pub struct Images {
    save: Handle<Image>,
    cancel: Handle<Image>,
    save_id: egui::TextureId,
    cancel_id: egui::TextureId,
}

const BTN_SIZE: (f32, f32) = (100., 40.);

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            save: asset_server.load("UI/save_and_exit.png"),
            cancel: asset_server.load("UI/cancel.png"),
            save_id: egui::TextureId::default(),
            cancel_id: egui::TextureId::default(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn controls_window(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut windows: ResMut<Windows>,
    player_controls: Query<&InputMap<PlayerAction>>,
    mut ui_state: ResMut<UIStateRes>,
    mut is_initialized: Local<bool>,
    mut images: Local<Images>,
    mut game_settings: ResMut<ReadWriteGameSettings>,
) {
    if !*is_initialized {
        *is_initialized = true;
        images.save_id = egui_context.add_image(images.save.clone_weak());
        images.cancel_id = egui_context.add_image(images.cancel.clone_weak());
    }

    let main_window = windows.get_primary().unwrap();
    let window_width_margin = egui_context.ctx_mut().style().spacing.window_margin.left * 2.0;

    let controls = player_controls.single();

    Window::new("Settings")
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .frame(Frame {
            fill: Color32::from_rgb(27, 51, 60),
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
                .striped(true)
                .min_col_width(ui.available_width() / COLUMNS_COUNT as f32 - window_width_margin)
                .show(ui, |ui| {
                    for action in PlayerAction::variants() {
                        ui.label(action.to_string());
                        let inputs = controls.get(action);
                        for index in 0..INPUT_VARIANTS {
                            let button_text = match inputs.get_at(index) {
                                Some(UserInput::Single(InputButton::Gamepad(gamepad_button))) => {
                                    format!("🎮 {:?}", gamepad_button)
                                }
                                Some(UserInput::Single(InputButton::Keyboard(keycode))) => {
                                    format!("🖮 {:?}", keycode)
                                }
                                Some(UserInput::Single(InputButton::Mouse(mouse_button))) => {
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
                let cancel = ui.add(egui::ImageButton::new(images.cancel_id, btn_size)).clicked();
                ui.visuals_mut().widgets.inactive.expansion = 0.;   // end bug fix

                if return_to_menu {
                    // first save to struct
                    game_settings.actual_settings = game_settings.pending_settings;
                    
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

                if cancel {
                    // reset settings
                    game_settings.pending_settings = game_settings.actual_settings;
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
    mut player_controls: Query<&mut InputMap<PlayerAction>>,
) {
    let mut controls = player_controls.single_mut();

    let mut active_binding = match active_binding {
        Some(active_binding) => active_binding,
        None => return,
    };

    Window::new(format!("Binding \"{}\"", active_binding.action))
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .show(egui.ctx_mut(), |ui| {
            if let Some(conflict) = &active_binding.conflict {
                ui.label(format!(
                    "Input \"{}\" is already used by \"{}\"",
                    conflict.input_button, conflict.action
                ));
                ui.horizontal(|ui| {
                    if ui.button("Replace").clicked() {
                        controls.remove(conflict.action, conflict.input_button);
                        controls.insert_at(
                            conflict.input_button,
                            active_binding.action,
                            active_binding.index,
                        );
                        commands.remove_resource::<ActiveBinding>();
                    }
                    if ui.button("Cancel").clicked() {
                        commands.remove_resource::<ActiveBinding>();
                    }
                });
            } else {
                ui.label("Press any key now");
                if let Some(input_button) = input_events.input_button() {
                    let conflict_action = controls.iter().find_map(|(inputs, action)| {
                        if action != active_binding.action && inputs.contains(&input_button.into())
                        {
                            return Some(action);
                        }
                        None
                    });
                    if let Some(action) = conflict_action {
                        active_binding.conflict.replace(BindingConflict {
                            action,
                            input_button,
                        });
                    } else {
                        controls.insert_at(
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
    gamepad_events: EventReader<'w, 's, GamepadEvent>,
}

impl InputEvents<'_, '_> {
    fn input_button(&mut self) -> Option<InputButton> {
        if let Some(keyboard_input) = self.keys.iter().next() {
            if keyboard_input.state == ElementState::Released {
                if let Some(key_code) = keyboard_input.key_code {
                    return Some(key_code.into());
                }
            }
        }

        if let Some(mouse_input) = self.mouse_buttons.iter().next() {
            if mouse_input.state == ElementState::Released {
                return Some(mouse_input.button.into());
            }
        }

        if let Some(GamepadEvent(_, event_type)) = self.gamepad_events.iter().next() {
            if let GamepadEventType::ButtonChanged(button, strength) = event_type.to_owned() {
                if strength <= 0.5 {
                    return Some(button.into());
                }
            }
        }

        None
    }
}
