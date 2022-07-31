use bevy::{prelude::*, window::PresentMode};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_flycam::{NoCameraPlayerPlugin, FlyCam};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::*;
use derive_more::Display;
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike, InputManagerBundle,
};
mod lib;
use lib::Map;
mod input_management;
use input_management::{binding_window_system, controls_window, toggle_settings, InputSettings};

pub const HEIGHT: f32 = 900.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Display)]
pub enum PlayerAction {
    OpenKeyBinds,
    ToggleInspector,
    Scream,
    /// Selects the players Nexus
    SelectNexus,

    /// Selects the previous node that was selected if applicable
    SelectPrevNode,

    /// Selects the next node (after select previous has been used)
    SelectNextNode,

    /// Selects a replicator, when pressed again selects the next replicator in the list, loops back to the beginning of the list
    SelectReplicator,

    /// Opens the Qubit Dialog
    OpenQubitTradePanel,

    /// Triggers the Selected Recombinator
    TriggerRecombinator,

    OpenOptionsMenu,

    HotKey1,
    HotKey2,
    HotKey3,
    HotKey4,
    
    PanForward,
    PanBackwards,
    PanLeft,
    PanRight,
}

/// Actions initiated by a KeyPress
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Display)]
pub enum PlayerGameAction {

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
        .add_plugin(InputManagerPlugin::<PlayerGameAction>::default())
        .add_plugin(NoCameraPlayerPlugin)
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
        //Camera
        .run();
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn_bundle(InputManagerBundle::<PlayerAction> {
            input_map: InputMap::new([
                (KeyCode::Space, PlayerAction::Scream),
                (KeyCode::Escape, PlayerAction::OpenKeyBinds),
                (KeyCode::Grave, PlayerAction::ToggleInspector),
                (KeyCode::W, PlayerAction::PanForward),
                (KeyCode::S, PlayerAction::PanBackwards),
                (KeyCode::A, PlayerAction::PanLeft),
                (KeyCode::D, PlayerAction::PanRight),
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
fn camera_controls(actions: Query<&ActionState<PlayerAction>>, mut camera: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    let mut transform = camera.single_mut();
    let actions = actions.single();

    if actions.pressed(PlayerAction::PanForward) {
        let pos = transform.forward() * time.delta_seconds();
        transform.translation += pos;
    }
    if actions.pressed(PlayerAction::PanBackwards) {
        let pos = transform.forward() * time.delta_seconds();
        transform.translation -= pos;
    }
    if actions.pressed(PlayerAction::PanLeft) {
        let pos = transform.left() * time.delta_seconds();
        transform.translation += pos;
    }
    if actions.pressed(PlayerAction::PanRight) {
        let pos = transform.left() * time.delta_seconds();
        transform.translation -= pos;
    }
    
}

fn spawn_test_map(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let path = "assets/test_map.json";

    use std::fs::File;
    let file = File::open(path).unwrap();
    use std::io::BufReader;
    let reader = BufReader::new(file);

    let map: Map = serde_json::from_reader(reader).unwrap();

    let map_ent = commands
        .spawn_bundle(TransformBundle::default())
        .insert(Name::new("Map"))
        .id();
    let mut node_ents = Vec::default();
    for node in map.nodes.values() {
        node_ents.push(
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: 1.0,
                        subdivisions: 5,
                    })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_translation(node.position),
                    ..default()
                })
                .insert(Name::new("Node"))
                .id(),
        );
    }
    let mut vector_ents = Vec::default();
    for vector in map.vectors.iter() {
        let node_0 = map.nodes.get(&vector.0).unwrap();
        let node_1 = map.nodes.get(&vector.1).unwrap();

        let pos = (node_0.position + node_1.position) / 2.0;
        let mut transform = Transform::from_translation(pos).looking_at(node_1.position, Vec3::Y);
        transform.scale.z = Vec3::distance(node_0.position, node_1.position) * 2.0;

        vector_ents.push(
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube {
                        size: 0.5,
                    })),
                    material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
                    transform,
                    ..default()
                })
                .insert(Name::new("Vector"))
                .id(),
        );
    }
    commands.entity(map_ent).push_children(&node_ents);
    commands.entity(map_ent).push_children(&vector_ents);
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
