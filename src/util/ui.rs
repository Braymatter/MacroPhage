use bevy::app::App;
use bevy::prelude::{Plugin, ResMut};
use bevy_egui::egui::{Color32, Ui, Stroke, Rounding};
use bevy_egui::{egui, EguiContext};

pub struct CustomEguiStyle;
impl Plugin for CustomEguiStyle {
    fn build(&self, app: &mut App) {
        app.add_startup_system(set_default_style);
    }
}

fn set_default_style(mut egui_context: ResMut<EguiContext>) {
    let ctx = egui_context.ctx_mut();
    let mut style: egui::Style = (*ctx.style()).clone();

    style.visuals.widgets.noninteractive.bg_stroke = Stroke::from((1.0, Color32::from_rgb(50, 232, 214)));
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::from((2.0, Color32::WHITE));
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::from((2.0, Color32::from_rgb(50, 232, 214)));
    style.visuals.widgets.inactive.fg_stroke = Stroke::from((2.0, Color32::WHITE));
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(0, 38, 38);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(0.2, Color32::from_rgb(50, 232, 214));
    style.visuals.widgets.inactive.rounding = Rounding::none();
    style.visuals.widgets.active.fg_stroke = Stroke::from((2.0, Color32::WHITE));
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(0, 50, 75);
    style.visuals.widgets.active.bg_stroke = Stroke::none();
    style.visuals.widgets.hovered.fg_stroke = Stroke::from((2.0, Color32::from_rgb(50, 232, 214)));
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0, 91, 135);
    style.visuals.widgets.hovered.bg_stroke = Stroke::none();

    ctx.set_style(style);
}

pub fn set_ui_style(ui: &mut Ui) {
    ui.visuals_mut().widgets.inactive.expansion = 0.;
    ui.visuals_mut().widgets.active.expansion = 0.;
    ui.visuals_mut().widgets.hovered.expansion = 0.;
}

pub fn set_ui_style_none(ui: &mut Ui) {
    ui.visuals_mut().widgets.inactive.expansion = 0.;
    ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::none();
    ui.visuals_mut().widgets.inactive.rounding = Rounding::none();
    ui.visuals_mut().widgets.active.expansion = 0.;
    ui.visuals_mut().widgets.active.bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.active.bg_stroke = Stroke::none();
    ui.visuals_mut().widgets.hovered.expansion = 0.;
    ui.visuals_mut().widgets.hovered.bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::none();
}