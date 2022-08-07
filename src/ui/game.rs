use bevy_egui::{
    egui::{Align2},
    EguiContext,
};

use bevy::prelude::*;
use bevy_inspector_egui::egui;

use crate::{
    game::LevelManagerRes,
    util::camera::{CameraState, PlayerCamMarker},
};
use crate::util::ui::{set_ui_style, set_ui_style_none};

use super::{UIState, UIStateRes};

pub struct Images {
    main_menu: Handle<Image>,
    player_border: Handle<Image>,
    opponent_border: Handle<Image>,
    quibit_icon: Handle<Image>,
    hex_button: Handle<Image>,
    main_menu_id: egui::TextureId,
    player_border_id: egui::TextureId,
    opponent_border_id: egui::TextureId,
    quibit_icon_id: egui::TextureId,
    hex_button_id: egui::TextureId,
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            main_menu: asset_server.load("UI/exit_game_small.png"),
            player_border: asset_server.load("UI/player_border.png"),
            opponent_border: asset_server.load("UI/opponent_border.png"),
            quibit_icon: asset_server.load("UI/quibit.png"),
            hex_button: asset_server.load("UI/hex_button.png"),
            main_menu_id: egui::TextureId::default(),
            player_border_id: egui::TextureId::default(),
            opponent_border_id: egui::TextureId::default(),
            quibit_icon_id: egui::TextureId::default(),
            hex_button_id: egui::TextureId::default(),
        }
    }
}

pub fn game_hud(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UIStateRes>,
    mut player_cam: Query<(&mut CameraState, &PlayerCamMarker)>,
    mut level_manager: ResMut<LevelManagerRes>,
    mut is_initialized: Local<bool>,
    mut images: Local<Images>,
) {
    if !*is_initialized {
        *is_initialized = true;
        images.main_menu_id = egui_context.add_image(images.main_menu.clone_weak());
        images.player_border_id = egui_context.add_image(images.player_border.clone_weak());
        images.opponent_border_id = egui_context.add_image(images.opponent_border.clone_weak());
        images.quibit_icon_id = egui_context.add_image(images.quibit_icon.clone_weak());
        images.hex_button_id = egui_context.add_image(images.hex_button.clone_weak());
    }
    let ctx = egui_context.ctx_mut();

    egui::Area::new("player avatar")
        .anchor(Align2::LEFT_TOP, egui::vec2(0., 0.))
        .fixed_pos(egui::pos2(0., 0.))
        .interactable(false)
        .show(ctx, |ui| {
            ui.image(images.player_border_id, egui::vec2(175., 200.));
        });

    egui::Area::new("top bar")
        .anchor(Align2::CENTER_TOP, egui::vec2(0., 5.))
        .fixed_pos(egui::pos2(0., 0.))
        .interactable(true)
        .show(ctx, |ui| {
            set_ui_style(ui);
            ui.visuals_mut().widgets.inactive.expansion = -5.;  // bug with egui imagebutton padding
            let return_to_menu = ui.add(egui::ImageButton::new(images.main_menu_id, egui::vec2(100., 40.)));
            ui.visuals_mut().widgets.inactive.expansion = 0.;   // end bug fix

            // TODO: verify leaving the game or hide this until escape is pressed
            if return_to_menu.clicked() {
                let (mut cam_state, _) = player_cam.single_mut();

                cam_state.should_pan = false;
                cam_state.should_zoom = false;

                level_manager.current_level = None;
                ui_state.current_state = UIState::MainMenu;
            }
        });

    egui::Area::new("opponent avatar")
        .anchor(Align2::RIGHT_TOP, egui::vec2(0., 0.))
        .fixed_pos(egui::pos2(0., 0.))
        .interactable(false)
        .show(ctx, |ui| {
            ui.image(images.opponent_border_id, egui::vec2(175., 200.));
        });

    egui::Area::new("ability_1")
        .anchor(Align2::LEFT_BOTTOM, egui::vec2(0., 0.))
        .fixed_pos(egui::pos2(0., 0.))
        .interactable(true)
        .show(ctx, |ui| {
            set_ui_style_none(ui);
            // TODO: hexes dont click well when overlapping
            // TODO: fork button to support hexagonal click area OR subset a small invisible button within.
            let btn = ui.add(egui::ImageButton::new(images.hex_button_id, egui::vec2(80., 100.)));
            if btn.clicked() {
                println!("Ability 1 used");
            }
        });

    egui::Area::new("ability_2")
        .anchor(Align2::LEFT_BOTTOM, egui::vec2(52., -76.))
        .fixed_pos(egui::pos2(0., 0.))
        .interactable(true)
        .show(ctx, |ui| {
            set_ui_style_none(ui);
            // TODO: hexes dont click well when overlapping
            // TODO: fork button to support hexagonal click area OR subset a small invisible button within.
            let btn = ui.add(egui::ImageButton::new(images.hex_button_id, egui::vec2(80., 100.)));
            if btn.clicked() {
                println!("Ability 2 used");
            }
        });

    egui::Area::new("ability_3")
        .anchor(Align2::LEFT_BOTTOM, egui::vec2(100., 0.))
        .fixed_pos(egui::pos2(0., 0.))
        .interactable(true)
        .show(ctx, |ui| {
            set_ui_style_none(ui);
            // TODO: hexes dont click well when overlapping
            // TODO: fork button to support hexagonal click area OR subset a small invisible button within.
            let btn = ui.add(egui::ImageButton::new(images.hex_button_id, egui::vec2(80., 100.)));
            if btn.clicked() {
                println!("Ability 3 used");
            }
        });

    egui::Area::new("resources")
        .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-16., -16.))
        .fixed_pos(egui::pos2(0., 0.))
        .interactable(true)
        .show(ctx, |ui| {
            ui.image(images.quibit_icon_id, egui::vec2(50., 50.));
        });
}
