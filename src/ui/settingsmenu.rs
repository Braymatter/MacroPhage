use bevy::{
    ecs::system::SystemParam,
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ElementState},
    prelude::*,
};
use bevy_egui::{egui::{Align2, Grid, Window}, egui, EguiContext};
use leafwing_input_manager::{prelude::*, user_input::InputButton};

use crate::{game::controller::PlayerAction, ui::UIState};
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
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            save: asset_server.load("UI/save_and_exit.png"),
        }
    }
}

pub fn controls_window(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    player_controls: Query<&InputMap<PlayerAction>>,
    mut ui_state: ResMut<UIStateRes>,
    images: Local<Images>,
) {
    let btn_size = egui::vec2(100., 40.);
    let save_and_return_btn = egui_context.add_image(images.save.clone());

    let main_window = windows.get_primary().unwrap();
    let window_width_margin = egui_context.ctx_mut().style().spacing.window_margin.left * 2.0;

    let controls = player_controls.single();

    Window::new("Settings")
        .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .default_width(main_window.width() - UI_MARGIN * 2.0 - window_width_margin)
        .show(egui_context.ctx_mut(), |ui| {
            set_ui_style(ui);

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
            let return_to_menu = ui.add(egui::ImageButton::new(save_and_return_btn, btn_size)).clicked();

            if return_to_menu {
                warn!("Should Save Settings Here");
                ui_state.current_state = UIState::MainMenu;
            }

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
