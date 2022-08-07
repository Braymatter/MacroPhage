use crate::game::settings::ReadWriteGameSettings;
use bevy::prelude::*;
//use bevy::render::camera::CameraTypePlugin;
use bevy_egui::{egui, EguiContext};

pub struct MouseCursorPlugin {}

pub struct MouseCursor {
    normal: Handle<Image>,
    clicked: Handle<Image>,
}

impl FromWorld for MouseCursor {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            normal: asset_server.load(MOUSE_PATH_NORMAL),
            clicked: asset_server.load(MOUSE_PATH_CLICKED),
        }
    }
}

#[derive(Component, Default)]
pub struct Mouse2dCamera {}

/// Texture data
pub const MOUSE_PATH_NORMAL: &str = "textures/mouse_normal.png";
pub const MOUSE_PATH_CLICKED: &str = "textures/mouse_clicked.png";
pub const MOUSE_SIZE: (f32, f32) = (32., 32.);
pub const MOUSE_OFFSET: (f32, f32) = (0., 0.);

/// Tracks the mouse on the screen and renders a cursor on top of its position.
/// This uses direct egui rendering due to its complexity
impl Plugin for MouseCursorPlugin {
    fn build(&self, app: &mut App) {
        app //.add_plugin(CameraTypePlugin::<Mouse2dCamera>::default())
            .add_system(move_cursor);
    }
}

fn move_cursor(
    mut egui_context: ResMut<EguiContext>,
    mut windows: ResMut<Windows>,
    mouse: Local<MouseCursor>,
    buttons: Res<Input<MouseButton>>,
    game_settings: Res<ReadWriteGameSettings>,
) {
    if !game_settings.actual_settings.use_hardware_mouse {
        let img = if buttons.any_pressed([MouseButton::Left, MouseButton::Right]) {
            egui_context.add_image(mouse.clicked.clone())
        } else {
            egui_context.add_image(mouse.normal.clone())
        };

        let ctx = egui_context.ctx_mut();
        let position = ctx
            .input()
            .pointer
            .hover_pos()
            .map(|coord| coord + egui::vec2(MOUSE_OFFSET.0, MOUSE_OFFSET.1));

        egui::Area::new("cursor")
            .fixed_pos(position.unwrap_or(egui::pos2(0., 0.)))
            .order(egui::Order::Tooltip)
            .interactable(false)
            .show(ctx, |ui| {
                ui.add(egui::Image::new(
                    img,
                    egui::vec2(MOUSE_SIZE.0, MOUSE_SIZE.1),
                ))
            });
    }

    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_visibility(game_settings.actual_settings.use_hardware_mouse);
}
