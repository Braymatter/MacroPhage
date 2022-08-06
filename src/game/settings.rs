use std::fs;
use std::fs::File;
use std::io::Read;
use bevy::prelude::*;
use bevy::window::WindowMode;
use directories::ProjectDirs;
use serde::Serialize;
use serde::Deserialize;

/// There are two kinds of settings in the game, the settings actually
/// applied to the game and ones that are pending to be applied. This is necessary
/// because egui forces us to track all values between frames ourselves.
#[derive(Default)]
pub struct ReadWriteGameSettings {
    pub actual_settings: GameSettings,
    pub(crate) pending_settings: GameSettings,
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
        fs::create_dir_all(path).unwrap_or_else(|_| eprintln!("Error creating directories on config path {}.", path.display()));
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