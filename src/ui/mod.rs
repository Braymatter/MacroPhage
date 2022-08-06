mod game;
mod gamelobby;
mod mainmenu;
mod settingsmenu;

pub mod mousecursor_egui;

use std::fs::{create_dir_all, File};
use std::io::Read;
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_egui::{egui, EguiContext};
use directories::ProjectDirs;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use serde::{Serialize, Deserialize};
use serde_json::Value;

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
    pub actual_settings: GameSettings,
    pending_settings: GameSettings,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub use_hardware_mouse: bool,
    pub music_enabled: bool,

    #[serde(with = "WindowModeDef")]
    pub window_display_mode: WindowMode,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "WindowMode")]
enum WindowModeDef {
    Windowed,
    BorderlessFullscreen,
    SizedFullscreen,
    Fullscreen,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            use_hardware_mouse: false,
            music_enabled: true,
            window_display_mode: WindowMode::Windowed,
        }
    }
}

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(load_settings)
            .add_system(changed_settings);
    }
}

fn load_settings(
    mut command: Commands
) {
    // deserialize the file we have
    if let Some(project_dirs) = ProjectDirs::from("", "", "macrophage") {
        let path = project_dirs.config_dir();
        create_dir_all(path).unwrap_or_else(|_| eprintln!("Error creating directories on config path {}.", path.display()));
        let file = File::open(path.join("settings.json"));
        match file {
            Ok(mut file) => {
                let mut buffer = String::new();
                let read_bytes = file.read_to_string(&mut buffer).unwrap_or(0);
                if read_bytes > 0 {
                    let settings: GameSettings = serde_json::from_str(&buffer).unwrap_or(GameSettings { ..default() });
                    command.insert_resource(ReadWriteGameSettings { actual_settings: settings, pending_settings: settings });

                    println!("Successfully loaded settings from settings.json: {}", buffer);
                }
            },
            Err(_) => println!("Couldn't access settings.json file; it may not exist yet."),
        };

    }

}

fn changed_settings(
    game_settings: ResMut<ReadWriteGameSettings>,
    mut windows: ResMut<Windows>,
) {
    if game_settings.is_changed() {
        windows.get_primary_mut().unwrap().set_mode(game_settings.actual_settings.window_display_mode);
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
