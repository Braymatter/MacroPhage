use bevy_egui::{
    egui::{Align2, Window},
    EguiContext,
};

use bevy::prelude::*;
use bevy_inspector_egui::egui;

use crate::util::{MapManifest, camera::{CameraState, PlayerCamMarker}};

use super::{UIState, UIStateRes};

pub struct LobbyStateRes{
    pub selected_map: String
}

pub fn lobby(mut egui_context: ResMut<EguiContext>, mut ui_state: ResMut<UIStateRes>, maps_manifest: ResMut<MapManifest>, mut lobby_state: ResMut<LobbyStateRes>, mut player_cam: Query<(&mut CameraState, &PlayerCamMarker)>) {
    Window::new("Game Lobby")
        .anchor(Align2::CENTER_CENTER, egui::vec2(0.0, -50.0))
        .show(egui_context.ctx_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui|{
                for map in &maps_manifest.map_files{
                    ui.radio_value(&mut lobby_state.selected_map, map.clone(), map);
                }
            });

            let play_btn = ui.button("Launch Game");
            let (mut cam_state, _) = player_cam.single_mut();

            if play_btn.clicked() && lobby_state.selected_map != *"".to_string(){ //Such unrust shall not stand!
                cam_state.should_pan = true;
                cam_state.should_zoom = true;
                ui_state.current_state = UIState::Game
            }

            let back_btn = ui.button("Main Menu");

            if back_btn.clicked() {
                ui_state.current_state = UIState::MainMenu;
            }
        });
}
