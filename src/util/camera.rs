use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy::ecs::system::QuerySingleError;
use bevy_mod_picking::PickingCameraBundle;
use leafwing_input_manager::prelude::ActionState;

use crate::game::controller::PlayerAction;

use super::mouse::MousePosition;

#[derive(Component, Debug)]
pub struct MacroPhageCamComp;

pub struct MacroCamPlugin;
impl Plugin for MacroCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera);
        app.add_system(pan_cam);
        app.add_system(pan_cam_mouse);
        app.add_system(zoom_cam);
        app.add_system(look_cam);
    }
}

#[derive(Component)]
pub struct CameraState {
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

impl Default for CameraState {
    fn default() -> Self {
        Self {
            pan_speed: 3.0,
            zoom_speed: 3.0,
            zoom_target_level: 5.0,
            look_target: Vec3::ZERO,
            should_pan: false,
            should_zoom: false,
            should_look_at: false,
        }
    }
}

#[derive(Component)]
pub struct PlayerCamMarker;
pub fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 15.0, 0.0).looking_at(Vec3::ZERO, -Vec3::X),
            ..default()
        })
        .insert(CameraState::default())
        .insert(PlayerCamMarker)
        .insert_bundle(PickingCameraBundle::default());
}

/// Could probably refactor these to be more generic, these systems assume a top down (y) view.
pub fn pan_cam(
    action_query: Query<&ActionState<PlayerAction>>,
    mut player_cam_query: Query<(&mut Transform, &CameraState)>,
    time: Res<Time>,
) {
    match action_query.get_single() {
        Ok(actions) => {
            for cam in player_cam_query.iter_mut() {
                let (mut transform, cam_state) = cam;
                if !cam_state.should_pan {
                    continue;
                }

                if actions.pressed(PlayerAction::PanLeft) {
                    let translation = transform.left() * cam_state.pan_speed * time.delta_seconds();
                    transform.translation += translation;
                }

                if actions.pressed(PlayerAction::PanRight) {
                    let translation = transform.right() * cam_state.pan_speed * time.delta_seconds();
                    transform.translation += translation;
                }

                if actions.pressed(PlayerAction::PanUp) {
                    let translation = transform.up() * cam_state.pan_speed * time.delta_seconds();
                    transform.translation += translation;
                }

                if actions.pressed(PlayerAction::PanDown) {
                    let translation = transform.down() * cam_state.pan_speed * time.delta_seconds();
                    transform.translation += translation;
                }
            }
        }
        Err(QuerySingleError::NoEntities(_)) => {
            println!("[Pan Cam] There is no ActionState loaded yet.");
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            panic!("[Pan Cam] Error: There is more than one ActionState!");
        }
    }


}

fn pan_cam_mouse(
    mut camera: Query<(&CameraState, &mut Transform)>,
    mouse_pos: Res<MousePosition>,
    time: Res<Time>,
) {
    let (state, mut transform) = camera.single_mut();

    let pan_radius = 0.05;
    //Controls speed boost at edge of screen
    let modifier = 5.0;

    if !state.should_pan {
        return;
    }

    if mouse_pos.ndc.x < -1.0 + pan_radius {
        //Gross but works
        let speed_modifier = -modifier * (mouse_pos.ndc.x + 1.0 - pan_radius) / pan_radius;
        let translation =
            transform.left() * state.pan_speed * time.delta_seconds() * speed_modifier;
        transform.translation += translation;
    }

    if mouse_pos.ndc.x > 1.0 - pan_radius {
        let speed_modifier = modifier * (mouse_pos.ndc.x - 1.0 + pan_radius) / pan_radius;
        let translation =
            transform.right() * state.pan_speed * time.delta_seconds() * speed_modifier;
        transform.translation += translation;
    }

    if mouse_pos.ndc.y > 1.0 - pan_radius {
        let speed_modifier = modifier * (mouse_pos.ndc.y - 1.0 + pan_radius) / pan_radius;
        let translation = transform.up() * state.pan_speed * time.delta_seconds() * speed_modifier;
        transform.translation += translation;
    }

    if mouse_pos.ndc.y < -1.0 + pan_radius {
        let speed_modifier = -modifier * (mouse_pos.ndc.y + 1.0 - pan_radius) / pan_radius;
        let translation =
            transform.down() * state.pan_speed * time.delta_seconds() * speed_modifier;
        transform.translation += translation;
    }
}

// TODO: Go back and rewrite this to lerp to a target y-level
pub fn zoom_cam(
    action_query: Query<&ActionState<PlayerAction>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut player_cam_query: Query<(&mut Transform, &CameraState)>,
    time: Res<Time>,
) {
    match action_query.get_single() {
        Ok(actions) => {
            for cam in player_cam_query.iter_mut() {
                let (mut transform, cam_state) = cam;

                if !cam_state.should_zoom {
                    continue;
                }

                let mut translation = Vec3::ZERO;

                if actions.pressed(PlayerAction::ZoomIn)
                    && transform.translation.y > cam_state.zoom_target_level
                {
                    translation += transform.forward() * (time.delta_seconds() * cam_state.zoom_speed);
                }

                if actions.pressed(PlayerAction::ZoomOut) {
                    translation += transform.back() * (time.delta_seconds() * cam_state.zoom_speed);
                }

                for wheel_event in mouse_wheel.iter() {
                    match wheel_event.unit {
                        bevy::input::mouse::MouseScrollUnit::Line => {
                            translation += transform.forward()
                                * wheel_event.y
                                * time.delta_seconds()
                                * cam_state.zoom_speed;
                        }
                        bevy::input::mouse::MouseScrollUnit::Pixel => {
                            error!("If you're seeing this reach out about your zooming experience!");
                            translation += transform.forward()
                                * wheel_event.y
                                * time.delta_seconds()
                                * cam_state.zoom_speed;
                        }
                    }
                }

                transform.translation += translation;

                //Correct back to minimum.
                if transform.translation.y < cam_state.zoom_target_level {
                    transform.translation.y = cam_state.zoom_target_level
                }
            }
        }
        Err(QuerySingleError::NoEntities(_)) => {
            println!("[Zoom Cam] There is no ActionState loaded yet.");
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            panic!("[Zoom Cam] Error: There is more than one ActionState!");
        }
    }
}

pub fn look_cam() {}
