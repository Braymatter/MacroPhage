use bevy::ecs::query::QuerySingleError;
use bevy::{prelude::*, window::PresentMode};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::*;
use leafwing_input_manager::{plugin::InputManagerPlugin, prelude::ActionState};
use macrophage::game::settings::{ReadWriteGameSettings, SettingsPlugin};
use macrophage::{
    audio::GameAudioPlugin,
    game::{
        controller::PlayerAction,
        mutationinput::{mutation_input, mutation_selection, MutationSelection},
        PlayerMutationEvent,
    },
    game::{map::spawn_map, LevelManagerRes},
    ui::mousecursor_egui::MouseCursorPlugin,
    ui::UIStatePlugin,
    util::{camera::MacroCamPlugin, MacroUtils},
};
use std::f32::consts::PI;

pub const HEIGHT: f32 = 900.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    let mut app = App::new();
    app
        //Bevy setup
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            width: HEIGHT * RESOLUTION,
            height: HEIGHT,
            title: "Macro:Phage".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: true,
            ..Default::default()
        })
        .insert_resource(LevelManagerRes {
            current_level: None,
        })
        .init_resource::<ReadWriteGameSettings>()
        .add_plugin(SettingsPlugin)
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
        //Mod picking
        .add_plugins(DefaultPickingPlugins)
        //Input management and remapping (TODO move to plugin)
        .add_plugin(InputManagerPlugin::<PlayerAction>::default())
        //.add_plugin(NoCameraPlayerPlugin)
        .add_plugin(MacroUtils {})
        //Test scene spawning
        .add_system(spawn_map)
        .add_system(macrophage::game::spawn_player)
        //Things that need keybinds should load after the player
        .add_plugin(MacroCamPlugin {})
        .add_system(toggle_inspector)
        //TODO move me to where I belong, game runner is not currently added
        .add_event::<PlayerMutationEvent>()
        .init_resource::<MutationSelection>()
        .add_system(mutation_selection)
        .add_system(mutation_input)
        .add_startup_system(create_background)
        //Audio
        .add_plugin(GameAudioPlugin);
    app.run();
}

fn create_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("background.png")),
        unlit: true,
        ..default()
    });
    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(200., 200.))));
    commands.spawn_bundle(PbrBundle {
        mesh: quad_handle,
        material: material_handle,
        transform: Transform::from_xyz(0., -20., 0.0)
            .with_rotation(Quat::from_axis_angle(Vec3::X, -PI / 2.0)),
        ..default()
    });
}

fn toggle_inspector(
    mut window_params: ResMut<WorldInspectorParams>,
    actions: Query<&ActionState<PlayerAction>>,
) {
    match actions.get_single() {
        Ok(action) => {
            if action.just_pressed(PlayerAction::ToggleInspector) {
                window_params.enabled = !window_params.enabled
            }
        }
        Err(QuerySingleError::NoEntities(_)) => {
            println!("[ToggleInspector] There is no ActionState loaded yet.");
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            panic!("[ToggleInspector] Error: There is more than one ActionState!");
        }
    }
}
