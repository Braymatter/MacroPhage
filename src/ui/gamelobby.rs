use bevy_egui::{
    egui::{Align2, Window},
    EguiContext,
};

use bevy::prelude::*;
use bevy_inspector_egui::egui;

use crate::{
    game::LevelManagerRes,
    util::{
        camera::{CameraState, PlayerCamMarker},
        MapManifest,
    },
};
use crate::util::ui::set_ui_style;

use super::{UIState, UIStateRes};

const UI_MARGIN: f32 = 10.0;

pub struct Images {
    launch: Handle<Image>,
    main_menu: Handle<Image>,
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            launch: asset_server.load("UI/launch.png"),
            main_menu: asset_server.load("UI/return.png"),
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
    images: Local<Images>,
) {
    let btn_size = egui::vec2(100., 40.);
    let launch_btn = egui_context.add_image(images.launch.clone());
    let main_menu_btn = egui_context.add_image(images.main_menu.clone());

    let main_window = windows.get_primary().unwrap();
    let window_width_margin = egui_context.ctx_mut().style().spacing.window_margin.left * 2.0;

    Window::new("Lobby")
        .anchor(Align2::CENTER_CENTER, egui::vec2(0.0, -50.0))
        .resizable(false)
        .collapsible(false)
        .default_width(main_window.width() - UI_MARGIN * 2.0 - window_width_margin)
        .show(egui_context.ctx_mut(), |ui| {
            set_ui_style(ui);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for map in &maps_manifest.map_files {
                    ui.radio_value(&mut lobby_state.selected_map, Some(map.clone()), map);
                }
            });

            ui.horizontal(|ui| {
                let play_btn = ui.add(egui::ImageButton::new(launch_btn, btn_size));

                let (mut cam_state, _) = player_cam.single_mut();

                if play_btn.clicked() && lobby_state.selected_map != None {
                    cam_state.should_pan = true;
                    cam_state.should_zoom = true;
                    ui_state.current_state = UIState::Game;
                    level_manager.current_level = lobby_state.selected_map.clone();
                } else if lobby_state.selected_map.clone() != level_manager.current_level {
                    level_manager.current_level = lobby_state.selected_map.clone();
                }

                let back_btn = ui.add(egui::ImageButton::new(main_menu_btn, btn_size));

                if back_btn.clicked() {
                    ui_state.current_state = UIState::MainMenu;
                }
            });

            ui.expand_to_include_rect(ui.available_rect_before_wrap());
        });
}
