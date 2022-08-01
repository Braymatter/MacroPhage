use std::{
    net::{SocketAddr, SocketAddrV4},
    str::FromStr,
};

use bevy::{prelude::*, window::PresentMode};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use bevy_mod_picking::*;
use bevy_punchthrough::client::{PunchthroughClientPlugin, PunchthroughEvent, RequestSwap};
use bevy_punchthrough::bevy_renet::{renet::RenetError};
use bevy_punchthrough::renet_plugin::PTRenetClientPlugin;
use macrophage::map::spawn_test_map;
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
pub enum PlayerGameAction {}

fn main() {
    let mut app = App::new();
    app
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
        .add_plugin(PTRenetClientPlugin)
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
        .add_startup_system(spawn_player)
        .add_system(monitor_punchthrough_events)
        .add_system(renet_test_controls)
        .add_system(panic_on_error_system)
        //Egui test
        .add_system(ui_example);

    build_punchthrough_plugin(&mut app);

    app.run();
}

fn build_punchthrough_plugin(app: &mut App) {
    let v4_socket = match SocketAddrV4::from_str("127.0.0.1:5001") {
        Ok(v4) => v4,
        Err(e) => {
            error!("Could not construct Local V4 Address {e:#?}");
            return;
        }
    };

    let local_socket: SocketAddr = SocketAddr::V4(v4_socket);

    let v4_pt_socket = match SocketAddrV4::from_str("127.0.0.1:5000") {
        Ok(sock) => sock,
        Err(e) => {
            error!("Error creating socket address for punchthrough service {e:#?}");
            return;
        }
    };

    let pt_socket = SocketAddr::V4(v4_pt_socket);

    app.add_plugin(PunchthroughClientPlugin {
        local_socket,
        punchthrough_server: pt_socket,
    });
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
                (KeyCode::Key1, PlayerAction::HotKey1),
                (KeyCode::Key2, PlayerAction::HotKey2),
                (KeyCode::Key3, PlayerAction::HotKey3),
                (KeyCode::Key4, PlayerAction::HotKey4),
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

fn renet_test_controls(
    actions: Query<&ActionState<PlayerAction>>,
    mut request_host_ev: EventWriter<RequestSwap>,
) {
    let actions = actions.single();
    if actions.just_pressed(PlayerAction::HotKey1) {
        println!("Requesting New Host Lobby");
        request_host_ev.send(RequestSwap::HostLobby);
    }
}

fn monitor_punchthrough_events(mut pt_events: EventReader<PunchthroughEvent>){
    for ev in pt_events.iter(){
        info!("Received PT Event {ev:#?}");
    }
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

// If any error is found we just panic
// This should probably change the game state to the main screen
// and open a pop up with a network error
fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        panic!("{}", e);
    }
}
