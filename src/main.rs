use bevy::{prelude::*, window::PresentMode};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::*;
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike, InputManagerBundle,
};

pub const HEIGHT: f32 = 900.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum PlayerAction {
    Scream,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: HEIGHT * RESOLUTION,
            height: HEIGHT,
            title: "MacroPhage".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(WorldInspectorParams {
            enabled: false,
            ..Default::default()
        })
        .add_system(toggle_inspector)
        .add_plugin(EguiPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(InputManagerPlugin::<PlayerAction>::default())
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_test_map)
        .add_startup_system(spawn_player)
        .add_system(ui_example)
        .run();
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn_bundle(InputManagerBundle::<PlayerAction> {
            input_map: InputMap::new([(KeyCode::Space, PlayerAction::Scream)]),
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
    input: ResMut<Input<KeyCode>>,
    mut window_params: ResMut<WorldInspectorParams>,
) {
    if input.just_pressed(KeyCode::Grave) {
        window_params.enabled = !window_params.enabled
    }
}
