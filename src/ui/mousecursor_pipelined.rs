use std::thread::sleep;
use std::time::Duration;
use bevy::core_pipeline::{AlphaMask3d, draw_2d_graph, draw_3d_graph, node, Opaque3d, Transparent2d, Transparent3d};
use bevy::prelude::*;
use bevy::render::camera::{ActiveCamera, Camera2d, CameraProjection, CameraTypePlugin, DepthCalculation, RenderTarget};
use bevy::render::primitives::Frustum;
use bevy::render::render_graph::{NodeRunError, RenderGraph, RenderGraphContext, SlotValue};
use bevy::render::render_graph::Node;
use bevy::render::{RenderApp, RenderStage};
use bevy::render::render_phase::RenderPhase;
use bevy::render::renderer::RenderContext;
use bevy::render::view::{RenderLayers, VisibleEntities};
use bevy::sprite::Anchor;
use bevy::window::WindowId;
use crate::util::mouse::MousePosition;

pub struct MouseCursorPlugin {}

#[derive(Component)]
pub struct MouseCursor {}
#[derive(Component, Default)]
pub struct Mouse2dCamera {}

/// The static name of the node for this graph pass in the two-pass render.
pub const MOUSE_PASS_DRIVER: &str = "mouse_pass_driver";

/// Texture data
pub const MOUSE_PATH: &str = "textures/Mouse.png";
pub const MOUSE_OFFSET: (f32, f32) = (-4., 0.);

/// Tracks the mouse on the screen and renders a cursor on top of its position.
/// This uses a separate orthgraphic 2d camera and two rendering phases to layer correctly in-game.
impl Plugin for MouseCursorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(CameraTypePlugin::<Mouse2dCamera>::default())
            .add_startup_system(load_mouse_cursor)
            .add_system(move_cursor);

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            let driver = MouseCameraDriver::new(&mut render_app.world);
            render_app.add_system_to_stage(RenderStage::Extract, extract_mouse_pass_camera_phases);

            let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();

            // render nodes in the correct order
            // this is what lets everything get layered correctly
            render_graph.add_node(MOUSE_PASS_DRIVER, driver);
            render_graph.add_node_edge(node::MAIN_PASS_DRIVER, MOUSE_PASS_DRIVER).unwrap();


            // critical to render this AFTER egui and any ui
            render_graph.add_node_edge(bevy_egui::node::EGUI_PASS, MOUSE_PASS_DRIVER).unwrap();
        }
    }
}

// Add 2D render phases for this camera
fn extract_mouse_pass_camera_phases(
    mut commands: Commands,
    active: Res<ActiveCamera<Mouse2dCamera>>,
) {
    if let Some(entity) = active.get() {
        commands.get_or_spawn(entity).insert_bundle((
            // Collect the phase items for 2d items for the rendering pass
            // this is a UI element, so only transparent2d is needed.
            RenderPhase::<Transparent2d>::default(),
        ));
    }
}

struct MouseCameraDriver {
    query: QueryState<Entity, With<Mouse2dCamera>>,
}

impl MouseCameraDriver {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}

/// There is nothing we need to do with the render world for a UI camera, but we
/// do want this to use the 2d rendering pipeline! The render node will run the rendering
/// sub graph for 2d objects since the mouse cursor is a 2d texture.
impl Node for MouseCameraDriver {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(&self, graph: &mut RenderGraphContext, _render_context: &mut RenderContext, world: &World) -> Result<(), NodeRunError> {
        for camera in self.query.iter_manual(world) {
            graph.run_sub_graph(draw_2d_graph::NAME, vec![SlotValue::Entity(camera)])?;
        }
        Ok(())
    }
}

fn load_mouse_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let far = 1000.0;
    let orthographic_projection = OrthographicProjection {
        far,
        depth_calculation: DepthCalculation::ZDifference,
        ..Default::default()
    };
    let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
    let view_projection =
        orthographic_projection.get_projection_matrix() * transform.compute_matrix().inverse();
    let frustum = Frustum::from_view_projection(
        &view_projection,
        &transform.translation,
        &transform.back(),
        orthographic_projection.far(),
    );
    let mouse_pass_layer = RenderLayers::layer(1);

    // The mouse UI camera is its own layer and objects associated to its layer
    // will all render together
    commands.spawn_bundle(
        OrthographicCameraBundle::<Mouse2dCamera> {
            camera: Camera {
                target: RenderTarget::Window(WindowId::primary()),
                near: orthographic_projection.near,
                far: orthographic_projection.far,
                ..Default::default()
            },
            orthographic_projection,
            visible_entities: VisibleEntities::default(),
            frustum,
            transform,
            global_transform: Default::default(),
            marker: Mouse2dCamera {}
        }
    )
        .insert(mouse_pass_layer);

    let texture_handle = asset_server.load(MOUSE_PATH);

    commands.spawn_bundle(SpriteBundle {
        texture: texture_handle,
        sprite: Sprite {
            anchor: Anchor::TopLeft,
            custom_size: Option::from(Vec2::new(40., 40.)),
            ..default()
        },
        transform: Transform::from_xyz(100., 0., 0.),
        ..default()
    })
        .insert(MouseCursor{})
        .insert(mouse_pass_layer);
}

fn move_cursor(mouse_pos: ResMut<MousePosition>, mut windows: ResMut<Windows>, mut query: Query<(&mut Transform, &MouseCursor)>) {
    for (mut transform, _) in query.iter_mut() {
        transform.translation.x = (mouse_pos.pixel_pos.x) + (MOUSE_OFFSET.0);
        transform.translation.y = (mouse_pos.pixel_pos.y) - (MOUSE_OFFSET.1);
    }

    // TODO: base this off a "Use Hardware Mouse" setting
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_visibility(false);
}