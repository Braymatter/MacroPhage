use bevy::{prelude::*, window::PresentMode};
use bevy::window::WindowMode::BorderlessFullscreen;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::*;
use leafwing_input_manager::{plugin::InputManagerPlugin, prelude::ActionState};
use macrophage::{
    audio::GameAudioPlugin,
    game::controller::PlayerAction,
    game::{map::spawn_map, LevelManagerRes},
    ui::UIStatePlugin,
    ui::mousecursor_egui::MouseCursorPlugin,
    ui::GameSettings,
    util::{camera::MacroCamPlugin, MacroUtils},
};
use macrophage::ui::ReadWriteGameSettings;


pub const HEIGHT: f32 = 900.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    let mut app = App::new();
    app
        //Bevy setup
        .insert_resource(WindowDescriptor {
            width: HEIGHT * RESOLUTION,
            height: HEIGHT,
            title: "Macro:Phage".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: true,
            //mode: BorderlessFullscreen,
            ..Default::default()
        })
        .insert_resource(LevelManagerRes {
            current_level: None,
        })
        .init_resource::<ReadWriteGameSettings>()
        .add_plugin(UIStatePlugin)
        .add_plugins(DefaultPlugins)
        //Egui (must be before inspector)
        .add_plugin(EguiPlugin)
        .add_plugin(MouseCursorPlugin {})
        //Egui Inspector
        .insert_resource(WorldInspectorParams {
            enabled: false,
            ..Default::default()
        })
        .add_plugin(WorldInspectorPlugin::new())
        .add_system(toggle_inspector)
        //Mod picking
        .add_plugins(DefaultPickingPlugins)
        //Input management and remapping (TODO move to plugin)
        .add_plugin(InputManagerPlugin::<PlayerAction>::default())
        //.add_plugin(NoCameraPlayerPlugin)
        .add_plugin(MacroUtils {})
        .add_plugin(MacroCamPlugin {})
        //Test scene spawning
        .add_system(spawn_map)
        .add_startup_system(macrophage::game::spawn_player)
        //Audio
        .add_plugin(GameAudioPlugin);
    app.run();
}

fn toggle_inspector(
    mut window_params: ResMut<WorldInspectorParams>,
    actions: Query<&ActionState<PlayerAction>>,
) {
    let actions = actions.single();

    if actions.just_pressed(PlayerAction::ToggleInspector) {
        window_params.enabled = !window_params.enabled
    }
}
