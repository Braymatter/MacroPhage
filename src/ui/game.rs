use bevy_egui::{
    egui::{Align2, Window},
    EguiContext,
};

use bevy::prelude::*;
use bevy_egui::egui::{Color32, Frame};
use bevy_inspector_egui::egui;

use crate::{
    game::LevelManagerRes,
    util::camera::{CameraState, PlayerCamMarker},
};
use crate::util::ui::set_ui_style;

use super::{UIState, UIStateRes};

const UI_MARGIN: f32 = 0.0;

pub struct Images {
    main_menu: Handle<Image>,
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            main_menu: asset_server.load("UI/exit_game_small.png"),
        }
    }
}

pub fn game_hud(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    mut ui_state: ResMut<UIStateRes>,
    mut player_cam: Query<(&mut CameraState, &PlayerCamMarker)>,
    mut level_manager: ResMut<LevelManagerRes>,
    images: Local<Images>,
) {
    let btn_size = egui::vec2(100., 40.);
    let main_menu_btn = egui_context.add_image(images.main_menu.clone());

    let main_window = windows.get_primary().unwrap();
    let window_width_margin = egui_context.ctx_mut().style().spacing.window_margin.left * 2.0;
    let window_height_margin = egui_context.ctx_mut().style().spacing.window_margin.top * 2.0;

    Window::new("MacroPhage HUD")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(Frame {
            fill: Color32::TRANSPARENT,
            ..default()
        })
        .default_width(main_window.width() - UI_MARGIN * 2.0 - window_width_margin)
        .default_height(main_window.height() - UI_MARGIN * 2.0 - window_height_margin)
        .anchor(Align2::CENTER_TOP, egui::vec2(0.0, 0.0))
        .show(egui_context.ctx_mut(), |ui| {
            set_ui_style(ui);
            let return_to_menu = ui.add(egui::ImageButton::new(main_menu_btn, btn_size)).clicked();

            if return_to_menu {
                let (mut cam_state, _) = player_cam.single_mut();

                cam_state.should_pan = false;
                cam_state.should_zoom = false;

                level_manager.current_level = None;
                ui_state.current_state = UIState::MainMenu;
            }

            ui.expand_to_include_rect(ui.available_rect_before_wrap());
        });
}
