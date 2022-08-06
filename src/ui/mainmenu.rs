use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContext};
use crate::util::ui::set_ui_style;

use super::{UIState, UIStateRes};

pub struct Images {
    play: Handle<Image>,
    join: Handle<Image>,
    settings: Handle<Image>,
    exit: Handle<Image>,
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            play: asset_server.load("UI/play.png"),
            join: asset_server.load("UI/join_game.png"),
            settings: asset_server.load("UI/settings.png"),
            exit: asset_server.load("UI/exit_game.png"),
        }
    }
}

pub fn main_menu(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UIStateRes>,
    mut exit_writer: EventWriter<AppExit>,
    images: Local<Images>,
) {
    let btn_size = egui::vec2(200., 80.);
    let play_btn = egui_context.add_image(images.play.clone());
    let join_btn = egui_context.add_image(images.join.clone());
    let settings_btn = egui_context.add_image(images.settings.clone());
    let exit_btn = egui_context.add_image(images.exit.clone());


    egui::Window::new("Main Menu")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -50.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(egui_context.ctx_mut(), |ui| {
            set_ui_style(ui);

            let play = ui.add(egui::ImageButton::new(play_btn, btn_size)).clicked();
            let join = ui.add(egui::ImageButton::new(join_btn, btn_size)).clicked();
            let settings = ui.add(egui::ImageButton::new(settings_btn, btn_size)).clicked();
            let exit = ui.add(egui::ImageButton::new(exit_btn, btn_size)).clicked();

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
