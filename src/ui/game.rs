use bevy_egui::{
    egui::{Align2, Window},
    EguiContext,
};

use bevy::prelude::*;
use bevy_inspector_egui::egui;

use crate::util::camera::{CameraState, PlayerCamMarker};

use super::{UIState, UIStateRes};


pub fn game_hud(mut egui_context: ResMut<EguiContext>, mut ui_state: ResMut<UIStateRes>, mut player_cam: Query<(&mut CameraState, &PlayerCamMarker)>){
    Window::new("MacroPhage HUD").anchor(Align2::CENTER_TOP, egui::vec2(0.0, 0.0)).show(egui_context.ctx_mut(), |ui| {
        let return_to_menu = ui.button("Exit To Main Menu").clicked();

        if return_to_menu {
            let (mut cam_state, _) = player_cam.single_mut();
            
            cam_state.should_pan = false;
            cam_state.should_zoom = false;

            ui_state.current_state = UIState::MainMenu;
        }
    });
}