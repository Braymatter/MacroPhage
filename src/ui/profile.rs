use std::fs::{create_dir_all, File};
use std::io::Write;
use bevy_egui::{
    egui::{Align2, Window},
    EguiContext,
};

use bevy::prelude::*;
use bevy_egui::egui::{Color32, Frame, RichText, Stroke};
use bevy_egui::egui::style::Margin;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::Align;
use directories::ProjectDirs;

use crate::game::settings::{PhageVariant, ReadWriteGameSettings};
use crate::ui::phage_select::RenderedPhage;
use crate::util::ui::set_ui_style;

use super::{UIState, UIStateRes};

const UI_MARGIN: f32 = 10.0;
const BTN_SIZE: (f32, f32) = (100., 40.);

pub struct Images {
    accept: Handle<Image>,
    accept_id: egui::TextureId,
    left: Handle<Image>,
    left_id: egui::TextureId,
    right: Handle<Image>,
    right_id: egui::TextureId,
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            accept: asset_server.load("UI/accept.png"),
            accept_id: egui::TextureId::default(),
            left: asset_server.load("UI/left_arrow.png"),
            left_id: egui::TextureId::default(),
            right: asset_server.load("UI/right_arrow.png"),
            right_id: egui::TextureId::default(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn profile(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    mut ui_state: ResMut<UIStateRes>,
    mut game_settings: ResMut<ReadWriteGameSettings>,
    mut is_initialized: Local<bool>,
    rendered_phage: Res<RenderedPhage>,
    mut images: Local<Images>,
) {
    if !*is_initialized {
        *is_initialized = true;
        images.accept_id = egui_context.add_image(images.accept.clone_weak());
        images.left_id = egui_context.add_image(images.left.clone_weak());
        images.right_id = egui_context.add_image(images.right.clone_weak());
    }

    let main_window = windows.get_primary().unwrap();
    let window_width_margin = egui_context.ctx_mut().style().spacing.window_margin.left * 2.0;
    let window_height_margin = egui_context.ctx_mut().style().spacing.window_margin.top * 2.0;

    let profile = Window::new(RichText::new("Profile").color(Color32::WHITE).size(32.))
        .anchor(Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .frame(Frame {
            fill: Color32::from_rgb(0, 38, 38),
            inner_margin: Margin::same(8.0),
            stroke: Stroke::new(0.6, Color32::from_rgb(50, 232, 214)),
            ..default()
        })
        .default_height(main_window.height() - UI_MARGIN * 20.0 - window_height_margin)
        .default_width(main_window.width() - UI_MARGIN * 2.0 - window_width_margin);

    let rendered_phage_id = egui_context.add_image(rendered_phage.image.clone());
    let ctx = egui_context.ctx_mut();

    profile.show(ctx, |ui| {
        set_ui_style(ui);
        let btn_size = egui::vec2(BTN_SIZE.0, BTN_SIZE.1);

        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .frame(Frame {
                fill: Color32::TRANSPARENT,
                ..default()
            }).show_inside(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("Player name ");
                    ui.text_edit_singleline(&mut game_settings.pending_profile.name);
                });
            });

        egui::SidePanel::left("left_panel")
            .resizable(false)
            .frame(Frame {
                fill: Color32::TRANSPARENT,
                ..default()
            })
            .width_range(40.0..=f32::INFINITY).show_inside(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(), |ui| {
                    ui.visuals_mut().widgets.inactive.expansion = -5.;  // bug with egui imagebutton padding
                    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::none();
                    let left_btn = ui.add(egui::ImageButton::new(images.left_id, egui::vec2(40., 100.)));
                    ui.visuals_mut().widgets.inactive.expansion = 0.;   // end bug fix

                    if left_btn.clicked() {
                        game_settings.pending_profile.phage = change_phage(game_settings.pending_profile.phage, Dir::Backward);
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(Frame {
                fill: Color32::TRANSPARENT,
                ..default()
            }).show_inside(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.visuals_mut().widgets.noninteractive.fg_stroke = Stroke::from((1.0, Color32::from_rgb(50, 232, 214)));
                    ui.image(rendered_phage_id, egui::vec2(768., 512.));
                });
            });

        egui::SidePanel::right("right_panel")
            .resizable(false)
            .frame(Frame {
                fill: Color32::TRANSPARENT,
                ..default()
            })
            .width_range(40.0..=f32::INFINITY).show_inside(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(), |ui| {
                    ui.visuals_mut().widgets.inactive.expansion = -5.;  // bug with egui imagebutton padding
                    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::none();
                    let right_btn = ui.add(egui::ImageButton::new(images.right_id, egui::vec2(40., 100.)));
                    ui.visuals_mut().widgets.inactive.expansion = 0.;   // end bug fix

                    if right_btn.clicked() {
                        game_settings.pending_profile.phage = change_phage(game_settings.pending_profile.phage, Dir::Forward);
                    }
                });
            });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .frame(Frame {
                fill: Color32::TRANSPARENT,
                ..default()
            }).show_inside(ui, |ui| {
                if game_settings.pending_profile.name.is_empty() {
                    ui.set_enabled(false);
                } else {
                    ui.set_enabled(true);
                }

                ui.visuals_mut().widgets.inactive.expansion = -5.;  // bug with egui imagebutton padding
                let accept_btn = ui.add(egui::ImageButton::new(images.accept_id, btn_size));
                ui.visuals_mut().widgets.inactive.expansion = 0.;   // end bug fix

                if accept_btn.clicked() {
                    // first save to struct
                    game_settings.actual_profile = game_settings.pending_profile.clone();

                    // now serialize to file
                    let json = serde_json::to_string(&game_settings.actual_profile);
                    if let Some(project_dirs) = ProjectDirs::from("", "", "macrophage") {
                        let path = project_dirs.config_dir();
                        create_dir_all(path).unwrap_or_else(|_| eprintln!("Error creating directories on config path {}.", path.display()));
                        let file = File::create(path.join("profile.json"));
                        match file {
                            Ok(mut file) => file.write_all(json.unwrap().as_bytes()).unwrap_or_else(|_| eprintln!("File write error on profile.json!")),
                            Err(_) => eprintln!("Error accessing profile.json file; it may be open in another program."),
                        };

                    }

                    ui_state.current_state = UIState::MainMenu;
                }
            });
    });
}

enum Dir {
    Forward,
    Backward,
}

fn change_phage(variant: PhageVariant, dir: Dir) -> PhageVariant {
    let list = vec![PhageVariant::Undecorated, PhageVariant::Antenna, PhageVariant::Cowboy, PhageVariant::Crown];

    let adjust: i32 = match dir {
        Dir::Forward => 1,
        Dir::Backward => -1
    };

    let cur = list.iter().position(|&it| it == variant).unwrap() as i32;
    let mut next = cur + adjust;

    if next == -1 {
        next = (list.len() - 1) as i32;
    } else if next == list.len() as i32 {
        next = 0;
    }

    list[next as usize]
}