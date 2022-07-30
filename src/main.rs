use bevy::{prelude::*, window::PresentMode};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::*;
use derive_more::Display;
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike, InputManagerBundle,
};

mod input_management;
use input_management::{binding_window_system, controls_window, toggle_settings, InputSettings};

pub const HEIGHT: f32 = 900.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Display)]
pub enum PlayerAction {
    OpenKeyBinds,
    ToggleInspector,
    Scream,
}

fn main() {
    App::new()
        //Bevy setup
        .insert_resource(WindowDescriptor {
            width: HEIGHT * RESOLUTION,
            height: HEIGHT,
            title: "MacroPhage".to_string(),
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
        .insert_resource(InputSettings::default())
        .add_system(controls_window)
        .add_system(binding_window_system)
        .add_system(toggle_settings)
        //Test scene spawning
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_test_map)
        .add_startup_system(spawn_player)
        //Egui test
        .add_system(ui_example)
        .run();
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn_bundle(InputManagerBundle::<PlayerAction> {
            input_map: InputMap::new([
                (KeyCode::Space, PlayerAction::Scream),
                (KeyCode::Escape, PlayerAction::OpenKeyBinds),
                (KeyCode::Grave, PlayerAction::ToggleInspector),
            ]),
            ..default()
        })
        .insert(Name::new("Player"));
}

fn ui_example(mut egui_context: ResMut<EguiContext>, actions: Query<&ActionState<PlayerAction>>) {
    let actions = actions.single();

    if actions.pressed(PlayerAction::Scream) {
        egui::Window::new("AHHHH").show(egui_context.ctx_mut(), |ui| {
            ui.label("AHHHH");
        });
    }
}

fn spawn_test_map(mut commands: Commands, assets: Res<AssetServer>) {
    let space_ship = assets.load("SpaceShip.gltf#Scene0");
    let asteroid = assets.load("VoxelAsteroid.gltf#Scene0");

    commands
        .spawn_bundle(TransformBundle::default())
        .insert(Name::new("Test Scene"))
        .with_children(|commands| {
            // I hate this nesting
            commands
                .spawn_bundle(TransformBundle::default())
                .insert(Name::new("SpaceShip"))
                .with_children(|commands| {
                    commands.spawn_scene(space_ship);
                });
            commands
                .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
                    -15., 15., 15.,
                )))
                .insert(Name::new("Asteroid"))
                .with_children(|commands| {
                    commands.spawn_scene(asteroid);
                });

            commands
                .spawn_bundle(PointLightBundle {
                    point_light: PointLight {
                        intensity: 500000.0,
                        range: 100.0,
                        shadows_enabled: true,
                        ..default()
                    },
                    transform: Transform::from_xyz(54.0, 58.0, 54.0),
                    ..default()
                })
                .insert(Name::new("Light"));

            commands
                .spawn_bundle(PointLightBundle {
                    point_light: PointLight {
                        intensity: 300000.0,
                        range: 100.0,
                        shadows_enabled: true,
                        ..default()
                    },
                    transform: Transform::from_xyz(-54.0, 58.0, -54.0),
                    ..default()
                })
                .insert(Name::new("Light"));
        });
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(60.0, 60.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
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
