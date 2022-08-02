use bevy::prelude::*;
use bevy_mod_picking::PickingCameraBundle;
use iyes_loopless::prelude::*;

#[derive(Component, Debug)]
pub struct MacroPhageCamComp;

pub struct MacroCamPlugin;
impl Plugin for MacroCamPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraStateRes::default());
        app.add_startup_system(spawn_camera);
        app.add_system(pan_cam.run_if(is_cam_pan));
        app.add_system(zoom_cam.run_if(is_cam_zoom));
        app.add_system(look_cam.run_if(is_cam_look));
    }
}

pub struct CameraStateRes {
    /// When How fast in units/sec to pan in a given direction
    pub pan_speed: f32,

    ///How much closer to the board to zoom for each mouse wheel movement
    pub zoom_speed: f32,

    ///The minimum y-value the camera can zoom to
    pub zoom_target_level: f32,

    ///The target to update the cameras orientation to look at each frame
    pub look_target: Vec3,

    pub should_pan: bool,
    pub should_zoom: bool,
    pub should_look_at: bool,
}

impl Default for CameraStateRes {
    fn default() -> Self {
        Self {
            pan_speed: 1.0,
            zoom_speed: 1.0,
            zoom_target_level: 0.0,
            look_target: Vec3::ZERO,
            should_pan: false,
            should_zoom: false,
            should_look_at: false,
        }
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 15.0, 0.0).looking_at(Vec3::ZERO, Vec3::X),
            ..default()
        })
        .insert_bundle(PickingCameraBundle::default());
}

pub fn is_cam_pan(cam_state: Res<CameraStateRes>) -> bool {
    cam_state.should_pan
}

pub fn is_cam_zoom(cam_state: Res<CameraStateRes>) -> bool {
    cam_state.should_zoom
}

pub fn is_cam_look(cam_state: Res<CameraStateRes>) -> bool {
    cam_state.should_look_at
}

pub fn pan_cam(){

}

pub fn zoom_cam(){

}

pub fn look_cam(){

}