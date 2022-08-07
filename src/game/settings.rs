use crate::game::controller::PlayerAction;
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;
use bevy::window::WindowMode;
use directories::ProjectDirs;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::user_input::InputKind;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::Read;

/// There are two kinds of settings in the game, the settings actually
/// applied to the game and ones that are pending to be applied. This is necessary
/// because egui forces us to track all values between frames ourselves.
#[derive(Default)]
pub struct ReadWriteGameSettings {
    pub actual_settings: GameSettings,
    pub(crate) pending_settings: GameSettings,
    pub actual_profile: PlayerProfile,
    pub(crate) pending_profile: PlayerProfile,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub use_hardware_mouse: bool,
    pub music_enabled: bool,

    #[serde(with = "WindowModeDef")]
    pub window_display_mode: WindowMode,

    pub inputs: InputMap<PlayerAction>,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "WindowMode")]
enum WindowModeDef {
    Windowed,
    BorderlessFullscreen,
    SizedFullscreen,
    Fullscreen,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub name: String,
    pub phage: PhageVariant,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum PhageVariant {
    Undecorated,
    Antenna,
    Cowboy,
    Crown
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            use_hardware_mouse: false,
            music_enabled: true,
            window_display_mode: WindowMode::Windowed,
            inputs: InputMap::new([
                (KeyCode::Space, PlayerAction::Scream),
                (KeyCode::Escape, PlayerAction::OpenKeyBinds),
                (KeyCode::Grave, PlayerAction::ToggleInspector),
                (KeyCode::W, PlayerAction::PanUp),
                (KeyCode::S, PlayerAction::PanDown),
                (KeyCode::A, PlayerAction::PanLeft),
                (KeyCode::D, PlayerAction::PanRight),
                (KeyCode::Key1, PlayerAction::HotKey1),
                (KeyCode::Key2, PlayerAction::HotKey2),
                (KeyCode::Key3, PlayerAction::HotKey3),
                (KeyCode::Key4, PlayerAction::HotKey4),
                (KeyCode::PageUp, PlayerAction::ZoomIn),
                (KeyCode::PageDown, PlayerAction::ZoomOut),
                (KeyCode::Left, PlayerAction::PanLeft),
                (KeyCode::Right, PlayerAction::PanRight),
                (KeyCode::Up, PlayerAction::PanUp),
                (KeyCode::Down, PlayerAction::PanDown),
            ]),
        }
    }
}

impl Default for PlayerProfile {
    fn default() -> Self {
        PlayerProfile {
            name: "".parse().unwrap(),
            phage: PhageVariant::Undecorated,
        }
    }
}

pub struct SettingsPlugin;
impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_settings)
            .add_system(changed_settings);
    }
}

/// Loads the settings and player profile from disk.
fn load_settings(
    mut command: Commands
) {
    let mut settings = GameSettings{ ..default() };
    let mut profile = PlayerProfile{ ..default() };

    if let Some(project_dirs) = ProjectDirs::from("", "", "macrophage") {
        let path = project_dirs.config_dir();
        fs::create_dir_all(path).unwrap_or_else(|_| {
            eprintln!("Error creating directories on config path {}.", path.display())
        });

        // deserialize the settings file
        let file = File::open(path.join("settings.json"));
        match file {
            Ok(mut file) => {
                let mut buffer = String::new();
                let read_bytes = file.read_to_string(&mut buffer).unwrap_or(0);
                if read_bytes > 0 {
                    settings = serde_json::from_str(&buffer).unwrap_or(GameSettings { ..default() });
                    println!("Successfully loaded settings from settings.json: {}", buffer);
                }
            },
            Err(_) => {
                println!("Couldn't access settings.json file; it may not exist yet.")
            },
        };

        // deserialize the player profile file
        let file = File::open(path.join("profile.json"));
        match file {
            Ok(mut file) => {
                let mut buffer = String::new();
                let read_bytes = file.read_to_string(&mut buffer).unwrap_or(0);
                if read_bytes > 0 {
                    profile = serde_json::from_str(&buffer).unwrap_or(PlayerProfile { ..default() });
                    println!("Successfully loaded settings from profile.json: {}", buffer);
                }
            },
            Err(_) => {
                println!("Couldn't access profile.json file; it may not exist yet.")
            },
        };

        // load
        command.insert_resource(ReadWriteGameSettings {
            actual_settings: settings.clone(),
            pending_settings: settings,
            actual_profile: profile.clone(),
            pending_profile: profile,
        });
    }

}

fn changed_settings(
    game_settings: ResMut<ReadWriteGameSettings>,
    mut windows: ResMut<Windows>,
    mut player_controls: Query<&mut InputMap<PlayerAction>>,
    mut loaded: Local<bool>
) {
    if game_settings.is_changed() || ! *loaded {
        // change display mode if needed
        if windows.get_primary_mut().is_some() {
            let window = windows.get_primary_mut().unwrap();
            if game_settings.actual_settings.window_display_mode != window.mode() {
                window.set_mode(game_settings.actual_settings.window_display_mode);
            }
        }

        // change bindings
        match player_controls.get_single_mut() {
            Ok(mut controls) => {
                let inputs = game_settings.actual_settings.inputs.clone();
                *controls = inputs;
            }
            Err(QuerySingleError::NoEntities(_)) => {
                println!("[Changed Settings] There is no InputMap loaded yet.");
            }
            Err(QuerySingleError::MultipleEntities(_)) => {
                panic!("[Changed Settings] Error: There is more than one InputMap!");
            }
        }

        *loaded = true;
    }
}

//Stolen from: https://github.com/Leafwing-Studios/leafwing-input-manager/blob/main/examples/binding_menu.rs#L2
#[derive(Copy, Clone)]
pub struct ActiveBinding {
    pub action: PlayerAction,
    pub index: usize,
    pub conflict: Option<BindingConflict>,
}

impl ActiveBinding {
    pub fn new(action: PlayerAction, index: usize) -> Self {
        Self {
            action,
            index,
            conflict: None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct BindingConflict {
    pub action: PlayerAction,
    pub input_button: InputKind,
}
