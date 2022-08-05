use bevy::prelude::*;
//use bevy::render::camera::CameraTypePlugin;
use bevy_egui::{egui, EguiContext};

pub struct MouseCursorPlugin {}

pub struct MouseCursor {
    texture: Handle<Image>,
}

impl FromWorld for MouseCursor {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            texture: asset_server.load(MOUSE_PATH),
        }
    }
}

#[derive(Component, Default)]
pub struct Mouse2dCamera {}

/// Texture data

// TODO: properly scaled down image with not terrible filtering
// ask Fethur to export a 64x64
pub const MOUSE_PATH: &str = "textures/Mouse_Small.png";
pub const MOUSE_SIZE: (f32, f32) = (32., 32.);
pub const MOUSE_OFFSET: (f32, f32) = (-4., 0.);

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
) {
    let img = egui_context.add_image(mouse.texture.clone());
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

    // TODO: base this off a "Use Hardware Mouse" setting
    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_visibility(false);
    }
}
