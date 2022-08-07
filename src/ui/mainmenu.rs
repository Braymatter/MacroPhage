use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContext};
use bevy_egui::egui::{Color32, Frame};
use crate::util::ui::set_ui_style;

use super::{UIState, UIStateRes};

pub struct Images {
    play: Handle<Image>,
    join: Handle<Image>,
    settings: Handle<Image>,
    exit: Handle<Image>,
    play_id: egui::TextureId,
    join_id: egui::TextureId,
    settings_id: egui::TextureId,
    exit_id: egui::TextureId,
}

const BTN_SIZE: (f32, f32) = (200., 80.);

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            play: asset_server.load("UI/play.png"),
            join: asset_server.load("UI/join_game.png"),
            settings: asset_server.load("UI/settings.png"),
            exit: asset_server.load("UI/exit_game.png"),
            play_id: egui::TextureId::default(),
            join_id: egui::TextureId::default(),
            settings_id: egui::TextureId::default(),
            exit_id: egui::TextureId::default(),
        }
    }
}

pub fn main_menu(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UIStateRes>,
    mut exit_writer: EventWriter<AppExit>,
    mut is_initialized: Local<bool>,
    mut images: Local<Images>,
) {
    if !*is_initialized {
        *is_initialized = true;
        images.play_id = egui_context.add_image(images.play.clone_weak());
        images.join_id = egui_context.add_image(images.join.clone_weak());
        images.settings_id = egui_context.add_image(images.settings.clone_weak());
        images.exit_id = egui_context.add_image(images.exit.clone_weak());

    }
    egui::Window::new("Main Menu")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -50.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(Frame {
            fill: Color32::from_rgb(27, 51, 60),
            ..default()
        })
        .show(egui_context.ctx_mut(), |ui| {
            set_ui_style(ui);
            let btn_size = egui::vec2(BTN_SIZE.0, BTN_SIZE.1);

            ui.visuals_mut().widgets.inactive.expansion = -5.;  // bug with egui imagebutton padding
            let play = ui.add(egui::ImageButton::new(images.play_id, btn_size)).clicked();
            let join = ui.add(egui::ImageButton::new(images.join_id, btn_size)).clicked();
            let settings = ui.add(egui::ImageButton::new(images.settings_id, btn_size)).clicked();
            let exit = ui.add(egui::ImageButton::new(images.exit_id, btn_size)).clicked();
            ui.visuals_mut().widgets.inactive.expansion = 0.;   // end bug fix

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
