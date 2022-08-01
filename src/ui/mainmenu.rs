use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContext};

use super::{UIState, UIStateRes};

pub fn main_menu(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UIStateRes>,
    mut exit_writer: EventWriter<AppExit>,
) {
    egui::Window::new("Main Menu")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -50.0))
        .show(egui_context.ctx_mut(), |ui| {
            let play = ui.button("Play").clicked();
            let join = ui.button("Join Game").clicked();
            let settings = ui.button("Settings").clicked();
            let exit = ui.button("Exit").clicked();

            if play {
                ui_state.current_state = UIState::Lobby;
            }

            if join {
                ui_state.current_state = UIState::JoinLobby;
            }

            if settings {
                ui_state.current_state = UIState::Settings;
            }

            if exit {
                exit_writer.send(AppExit);
            }
        });
}
