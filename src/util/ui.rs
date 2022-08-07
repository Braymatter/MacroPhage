use bevy_egui::egui::{Color32, Ui, Stroke, Rounding};

pub fn set_ui_style(ui: &mut Ui) {
    ui.visuals_mut().widgets.inactive.expansion = 0.;
    ui.visuals_mut().widgets.inactive.bg_fill = Color32::from_rgb(27, 51, 60);
    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(0.2, Color32::from_rgb(36, 209, 183));
    ui.visuals_mut().widgets.inactive.rounding = Rounding::none();
    ui.visuals_mut().widgets.active.expansion = 0.;
    ui.visuals_mut().widgets.active.bg_fill = Color32::from_rgb(0, 50, 75);
    ui.visuals_mut().widgets.active.bg_stroke = Stroke::none();
    ui.visuals_mut().widgets.hovered.expansion = 0.;
    ui.visuals_mut().widgets.hovered.bg_fill = Color32::from_rgb(0, 91, 135);
    ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::none();
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