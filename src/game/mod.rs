use self::controller::PlayerAction;
use bevy::prelude::*;
use leafwing_input_manager::{prelude::*,};

pub mod controller;

pub fn spawn_player(mut commands: Commands) {
    commands
        .spawn_bundle(InputManagerBundle::<PlayerAction> {
            //TODO: Can we export this bundle from controller.rs?
            input_map: InputMap::new([
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
                (KeyCode::Down, PlayerAction::PanDown)
                
            ]),
            ..default()
        })
        .insert(Name::new("Player"));
}
