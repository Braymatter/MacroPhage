use bevy_egui::{
    egui::{Align2, Window},
    EguiContext,
};

use bevy::prelude::*;
use bevy_inspector_egui::egui;

use crate::{
    map::LevelManagerRes,
    util::{
        camera::{CameraState, PlayerCamMarker},
        MapManifest,
    },
};

use super::{UIState, UIStateRes};

pub struct LobbyStateRes {
    pub selected_map: Option<String>,
}

pub fn lobby(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UIStateRes>,
    maps_manifest: ResMut<MapManifest>,
    mut lobby_state: ResMut<LobbyStateRes>,
    mut player_cam: Query<(&mut CameraState, &PlayerCamMarker)>,
    mut level_manager: ResMut<LevelManagerRes>,
) {
    Window::new("Game Lobby")
        .anchor(Align2::CENTER_CENTER, egui::vec2(0.0, -50.0))
        .show(egui_context.ctx_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for map in &maps_manifest.map_files {
                    ui.radio_value(&mut lobby_state.selected_map, Some(map.clone()), map);
                }
            });

            let play_btn = ui.button("Launch Game");
            let (mut cam_state, _) = player_cam.single_mut();

            if play_btn.clicked() && lobby_state.selected_map != None {
                cam_state.should_pan = true;
                cam_state.should_zoom = true;
                ui_state.current_state = UIState::Game;
                level_manager.current_level = lobby_state.selected_map.clone();
            } else if lobby_state.selected_map.clone() != level_manager.current_level {
                level_manager.current_level = lobby_state.selected_map.clone();
            }

            let back_btn = ui.button("Main Menu");

            if back_btn.clicked() {
                ui_state.current_state = UIState::MainMenu;
            }
        });
}
