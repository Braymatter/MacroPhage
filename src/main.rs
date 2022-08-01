use bevy::{prelude::*, window::PresentMode};
use bevy_egui::{EguiPlugin};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::*;
use derive_more::Display;
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, },
    Actionlike,
};

use macrophage::{map::spawn_test_map, game::controller::PlayerAction};

mod input_management;
use input_management::{binding_window_system, controls_window, toggle_settings, InputSettings};

pub const HEIGHT: f32 = 900.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

/// Actions initiated by a KeyPress
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Display)]
pub enum PlayerGameAction {}

fn main() {
    let mut app = App::new();
    app
        //Bevy setup
        .insert_resource(WindowDescriptor {
            width: HEIGHT * RESOLUTION,
            height: HEIGHT,
            title: "Macro:Phage".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        //Egui (must be before inspector)
        .add_plugin(EguiPlugin)
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
        .add_plugin(NoCameraPlayerPlugin)
        .insert_resource(InputSettings::default())
        .add_system(controls_window)
        .add_system(binding_window_system)
        .add_system(toggle_settings)
        //Test scene spawning
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_test_map)
        .add_startup_system(macrophage::game::spawn_player);
    app.run();
}


fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(4.0, 5.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(FlyCam)
        .insert_bundle(PickingCameraBundle::default());
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

