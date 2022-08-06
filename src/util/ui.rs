use bevy_egui::egui::{Color32, Ui, Stroke};

pub fn set_ui_style(ui: &mut Ui) {
    ui.visuals_mut().widgets.inactive.expansion = -5.;
    ui.visuals_mut().widgets.inactive.bg_fill = Color32::BLACK;
    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::none();
    ui.visuals_mut().widgets.active.expansion = 0.;
    ui.visuals_mut().widgets.active.bg_fill = Color32::from_rgb(0, 50, 75);
    ui.visuals_mut().widgets.active.bg_stroke = Stroke::none();
    ui.visuals_mut().widgets.hovered.expansion = 0.;
    ui.visuals_mut().widgets.hovered.bg_fill = Color32::from_rgb(0, 91, 135);
    ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::none();
}