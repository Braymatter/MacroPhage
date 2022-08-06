mod game;
mod gamelobby;
mod mainmenu;
mod settingsmenu;

pub mod mousecursor_egui;

use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_egui::{egui, EguiContext};
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::game::controller::PlayerAction;

use self::{gamelobby::LobbyStateRes, settingsmenu::binding_window_system};

pub fn ui_example(
    mut egui_context: ResMut<EguiContext>,
    actions: Query<&ActionState<PlayerAction>>,
) {
    let actions = actions.single();

    if actions.pressed(PlayerAction::Scream) {
        egui::Window::new("AHHHH").show(egui_context.ctx_mut(), |ui| {
            ui.label("AHHHH");
        });
    }
}

pub enum UIState {
    MainMenu,
    Settings,
    Lobby,
    JoinLobby,
    JoiningLobby { lobby_id: String },
    Game,
}

pub struct UIStateRes {
    current_state: UIState,
}

/// There are two kinds of settings in the game, the settings actually
/// applied to the game and ones that are pending to be applied. This is necessary
/// because egui forces us to track all values between frames ourselves.
#[derive(Default)]
pub struct ReadWriteGameSettings {
    actual_settings: GameSettings,
    pending_settings: GameSettings,
}

#[derive(Copy, Clone)]
pub struct GameSettings {
    use_hardware_mouse: bool,
    window_display_mode: WindowMode,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            use_hardware_mouse: false,
            window_display_mode: WindowMode::Windowed,
        }
    }
}

pub struct UIStatePlugin;
impl Plugin for UIStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UIStateRes {
            current_state: UIState::MainMenu,
        })
        .insert_resource(LobbyStateRes { selected_map: None })
        .add_system(self::mainmenu::main_menu.run_if(show_main_menu))
        .add_system(self::settingsmenu::controls_window.run_if(show_settings_menu))
        .add_system(self::gamelobby::lobby.run_if(show_lobby_screen))
        .add_system(self::game::game_hud.run_if(show_game_hud))
        .add_system(binding_window_system);
    }
}

//Could probably write a macro to handle this (cries in first class fn languages)
fn show_lobby_screen(ui_state: Res<UIStateRes>) -> bool {
    matches!(&ui_state.current_state, UIState::Lobby)
}

fn show_main_menu(ui_state: Res<UIStateRes>) -> bool {
    matches!(&ui_state.current_state, UIState::MainMenu)
}

fn show_settings_menu(ui_state: Res<UIStateRes>) -> bool {
    matches!(&ui_state.current_state, UIState::Settings)
}

fn show_game_hud(ui_state: Res<UIStateRes>) -> bool {
    matches!(&ui_state.current_state, UIState::Game)
}
