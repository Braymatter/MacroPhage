use bevy_egui::{
    egui::{Align2, Window},
    EguiContext,
};

use bevy::prelude::*;
use bevy_egui::egui::style::Margin;
use bevy_egui::egui::{Color32, Frame, RichText, Stroke};
use bevy_inspector_egui::egui;

use crate::util::ui::set_ui_style;
use crate::{
    game::LevelManagerRes,
    util::{
        camera::{CameraState, PlayerCamMarker},
        MapManifest,
    },
};

use super::{UIState, UIStateRes};

const UI_MARGIN: f32 = 10.0;
const BTN_SIZE: (f32, f32) = (100., 40.);

pub struct Images {
    launch: Handle<Image>,
    main_menu: Handle<Image>,
    launch_id: egui::TextureId,
    main_menu_id: egui::TextureId,
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            launch: asset_server.load("UI/launch.png"),
            main_menu: asset_server.load("UI/return.png"),
            launch_id: egui::TextureId::default(),
            main_menu_id: egui::TextureId::default(),
        }
    }
}

pub struct LobbyStateRes {
    pub selected_map: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn lobby(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    mut ui_state: ResMut<UIStateRes>,
    maps_manifest: ResMut<MapManifest>,
    mut lobby_state: ResMut<LobbyStateRes>,
    mut player_cam: Query<(&mut CameraState, &PlayerCamMarker)>,
    mut level_manager: ResMut<LevelManagerRes>,
    mut is_initialized: Local<bool>,
    mut images: Local<Images>,
) {
    if !*is_initialized {
        *is_initialized = true;
        images.launch_id = egui_context.add_image(images.launch.clone_weak());
        images.main_menu_id = egui_context.add_image(images.main_menu.clone_weak());
    }

    let main_window = windows.get_primary().unwrap();
    let window_width_margin = egui_context.ctx_mut().style().spacing.window_margin.left * 2.0;

    let lobby = Window::new(RichText::new("Lobby").color(Color32::WHITE).size(32.))
        .anchor(Align2::CENTER_CENTER, egui::vec2(0.0, -50.0))
        .resizable(false)
        .collapsible(false)
        .frame(Frame {
            fill: Color32::from_rgb(0, 38, 38),
            inner_margin: Margin::same(8.0),
            stroke: Stroke::new(0.6, Color32::from_rgb(50, 232, 214)),
            ..default()
        })
        .default_width(main_window.width() - UI_MARGIN * 2.0 - window_width_margin);

    lobby.show(egui_context.ctx_mut(), |ui| {
        set_ui_style(ui);
        let btn_size = egui::vec2(BTN_SIZE.0, BTN_SIZE.1);

        egui::ScrollArea::vertical().show(ui, |ui| {
            for map in &maps_manifest.map_files {
                ui.radio_value(&mut lobby_state.selected_map, Some(map.clone()), map);
            }
        });

        ui.horizontal(|ui| {
            ui.visuals_mut().widgets.inactive.expansion = -5.; // bug with egui imagebutton padding
            let play_btn = ui.add(egui::ImageButton::new(images.launch_id, btn_size));
            let back_btn = ui.add(egui::ImageButton::new(images.main_menu_id, btn_size));
            ui.visuals_mut().widgets.inactive.expansion = 0.; // end bug fix

            let (mut cam_state, _) = player_cam.single_mut();

            if play_btn.clicked() && lobby_state.selected_map != None {
                cam_state.should_pan = true;
                cam_state.should_zoom = true;
                ui_state.current_state = UIState::Game;
                level_manager.current_level = lobby_state.selected_map.clone();
            } else if lobby_state.selected_map.clone() != level_manager.current_level {
                level_manager.current_level = lobby_state.selected_map.clone();
            }

            if back_btn.clicked() {
                ui_state.current_state = UIState::MainMenu;
            }
        });

        ui.expand_to_include_rect(ui.available_rect_before_wrap());
    });
}
