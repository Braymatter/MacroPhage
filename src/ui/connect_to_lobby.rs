use std::net::SocketAddr;

use bevy_egui::{
    egui::{Align2, Window},
    EguiContext,
};

use bevy::prelude::*;
use bevy_inspector_egui::egui::vec2;

use crate::net::ConnectRequestEvent;

use super::{UIState, UIStateRes};

pub fn show_connect_dialog(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UIStateRes>,
    mut join_requests: EventWriter<ConnectRequestEvent>,
) {
    Window::new("Connect To Server")
        .anchor(Align2::CENTER_CENTER, vec2(0.0, -50.0))
        .show(egui_context.ctx_mut(), |ui| {
            ui.text_edit_singleline(&mut ui_state.target_host);
            let conbtn = ui.button("Connect");
            let cancel = ui.button("Cancel");

            if cancel.clicked() {
                ui_state.current_state = UIState::MainMenu;
            }

            if conbtn.clicked() {
                match ui_state.target_host.parse::<SocketAddr>() {
                    Ok(addy) => {
                        join_requests.send(ConnectRequestEvent { socket: addy });
                        ui_state.current_state = UIState::JoiningLobby
                    }
                    Err(e) => {
                        error!("{}", e)
                    }
                }
            }
        });
}

pub fn show_connecting_dialog(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UIStateRes>,
) {
    Window::new("Connecting To Server")
        .anchor(Align2::CENTER_CENTER, vec2(0.0, -50.0))
        .show(egui_context.ctx_mut(), |ui| {
            let cancel = ui.button("Cancel");

            if cancel.clicked() {
                ui_state.current_state = UIState::JoinLobby;
            }
        });
}
