use std::f32::consts::PI;
use bevy::core_pipeline::{AlphaMask3d, draw_3d_graph, node, Opaque3d, RenderTargetClearColors, Transparent3d};
use bevy::prelude::*;
use bevy::render::camera::{ActiveCamera, CameraTypePlugin, RenderTarget};
use bevy::render::render_graph::{NodeRunError, RenderGraph, RenderGraphContext, SlotValue};
use bevy::render::render_graph::Node;
use bevy::render::{RenderApp, RenderStage};
use bevy::render::render_phase::RenderPhase;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::renderer::RenderContext;
use bevy::render::view::{RenderLayers};
use bevy::scene::InstanceId;
use iyes_loopless::prelude::*;
use crate::game::settings::{PhageVariant, ReadWriteGameSettings};
use crate::ui::phage_select::RenderGraphState::Unloaded;
use crate::ui::show_profile_screen;

pub struct PhageSelectPlugin;

#[derive(Component, Default)]
pub struct Phage3dCamera {}

pub struct RenderedPhage {
    pub image: Handle<Image>,
}

/// The static name of the node for this graph pass in the multi-pass render.
pub const PHAGE_PASS_DRIVER: &str = "phage_camera_pass_driver";

const PHAGE_PASS_LAYER: RenderLayers = RenderLayers::layer(1);

/// Helps render a phage in the phage selection window.
/// We need to use a different camera to render to a texture.
impl Plugin for PhageSelectPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(CameraTypePlugin::<Phage3dCamera>::default())
            .init_resource::<SceneInstance>()
            .init_resource::<PhageEngine>()
            .add_startup_system(setup)
            .add_system(phage_rotation.run_if(show_profile_screen))
            .add_system(selected_phage.run_if(show_profile_screen))
            .add_system(load.run_if(show_profile_screen))
            .add_system(unload.run_if_not(show_profile_screen))
            .add_system(tag_scene);

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            let driver = PhageCameraDriver::new(&mut render_app.world);
            render_app.add_system_to_stage(RenderStage::Extract, extract_phage_pass_camera_phases);

            let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
            render_graph.add_node(PHAGE_PASS_DRIVER, driver);
            render_graph.add_node_edge(node::MAIN_PASS_DEPENDENCIES, PHAGE_PASS_DRIVER).unwrap();
            render_graph.add_node_edge(node::CLEAR_PASS_DRIVER, PHAGE_PASS_DRIVER).unwrap();
            render_graph.add_node_edge(PHAGE_PASS_DRIVER, node::MAIN_PASS_DRIVER).unwrap();

            println!("[Phage Selector] initialized");
        }
    }
}

// Add render phases for this camera
fn extract_phage_pass_camera_phases(
    mut commands: Commands,
    active: Res<ActiveCamera<Phage3dCamera>>,
) {
    if let Some(entity) = active.get() {
        commands.get_or_spawn(entity).insert_bundle((
            RenderPhase::<Opaque3d>::default(),
            RenderPhase::<AlphaMask3d>::default(),
            RenderPhase::<Transparent3d>::default(),
        ));
    }
}

struct PhageCameraDriver {
    query: QueryState<Entity, With<Phage3dCamera>>,
}

impl PhageCameraDriver {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}

impl Node for PhageCameraDriver {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(&self, graph: &mut RenderGraphContext, _render_context: &mut RenderContext, world: &World) -> Result<(), NodeRunError> {
        for camera in self.query.iter_manual(world) {
            graph.run_sub_graph(draw_3d_graph::NAME, vec![SlotValue::Entity(camera)])?;
        }
        Ok(())
    }
}

// Track the scene id until loaded so we can associate it afterwards
#[derive(Default)]
struct SceneInstance {
    phage: Option<InstanceId>,
    antenna: Option<InstanceId>,
    cowboy: Option<InstanceId>,
    crown: Option<InstanceId>,
}

#[derive(Component)]
struct PhageWholeModel;

#[derive(Component)]
struct PhageModel;

#[derive(Component)]
struct AntennaModel;

#[derive(Component)]
struct CowboyModel;

#[derive(Component)]
struct CrownModel;

#[allow(clippy::too_many_arguments)]
fn setup(
    mut commands: Commands,
    mut clear_colors: ResMut<RenderTargetClearColors>,
    mut images: ResMut<Assets<Image>>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut scene_instance: ResMut<SceneInstance>,
    asset_server: Res<AssetServer>
) {
    let size = Extent3d {
        width: 768,
        height: 512,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };

    image.resize(size);
    let image_handle = images.add(image);
    commands.insert_resource(RenderedPhage {
        image: image_handle.clone()
    });

    let render_target = RenderTarget::Image(image_handle.clone());
    clear_colors.insert(render_target.clone(), Color::rgba(0., 0., 0., 0.));
    commands.spawn_bundle(PerspectiveCameraBundle::<Phage3dCamera> {
        camera: Camera {
            target: render_target,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(19.0, 27.0, 25.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..PerspectiveCameraBundle::new()
        })
        .insert(PHAGE_PASS_LAYER);

    commands.spawn_bundle(TransformBundle::from(Transform::from_xyz(0., 0., 0.)))
        .with_children(|p| {
            let phage_instance_id = scene_spawner.spawn_as_child(asset_server.load("meshes/Avatar Phage.glb#Scene0"), p.parent_entity());
            let antenna_instance_id = scene_spawner.spawn_as_child(asset_server.load("meshes/Antenna.glb#Scene0"), p.parent_entity());
            let cowboy_instance_id = scene_spawner.spawn_as_child(asset_server.load("meshes/CowboyHat.glb#Scene0"), p.parent_entity());
            let crown_instance_id = scene_spawner.spawn_as_child(asset_server.load("meshes/crown.glb#Scene0"), p.parent_entity());

            scene_instance.phage = Some(phage_instance_id);
            scene_instance.antenna = Some(antenna_instance_id);
            scene_instance.cowboy = Some(cowboy_instance_id);
            scene_instance.crown = Some(crown_instance_id);
        })
        .insert(PHAGE_PASS_LAYER)
        .insert(PhageWholeModel);

    // Warning: lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..Default::default()
    });
}

/// Attaching entities to scenes is convoluted because the scenes
/// must first be loaded.
/// see: https://github.com/bevyengine/bevy/blob/v0.7.0/examples/3d/update_gltf_scene.rs
/// for technical details on manipulating the scenes.
fn tag_scene(
    mut commands: Commands,
    scene_spawner: Res<SceneSpawner>,
    scene_instance: Res<SceneInstance>,

    // TODO: shove data into a vec so this isn't absolutely disgusting
    mut phage_loaded: Local<bool>,
    mut antenna_loaded: Local<bool>,
    mut cowboy_loaded: Local<bool>,
    mut crown_loaded: Local<bool>,
) {
    if !*phage_loaded {
        if let Some(instance_id) = scene_instance.phage {
            if let Some(entity_iter) = scene_spawner.iter_instance_entities(instance_id) {
                entity_iter.for_each(|entity| {
                    commands.entity(entity).insert(PHAGE_PASS_LAYER);
                    commands.entity(entity).insert(PhageModel);
                });

                *phage_loaded = true;
            }
        }
    }

    if !*antenna_loaded {
        if let Some(instance_id) = scene_instance.antenna {
            if let Some(entity_iter) = scene_spawner.iter_instance_entities(instance_id) {
                entity_iter.for_each(|entity| {
                    commands.entity(entity).insert(PHAGE_PASS_LAYER);
                    commands.entity(entity).insert(AntennaModel);
                });

                *antenna_loaded = true;
            }
        }
    }

    if !*cowboy_loaded {
        if let Some(instance_id) = scene_instance.cowboy {
            if let Some(entity_iter) = scene_spawner.iter_instance_entities(instance_id) {
                entity_iter.for_each(|entity| {
                    commands.entity(entity).insert(PHAGE_PASS_LAYER);
                    commands.entity(entity).insert(CowboyModel);
                });

                *cowboy_loaded = true;
            }
        }
    }

    if !*crown_loaded {
        if let Some(instance_id) = scene_instance.crown {
            if let Some(entity_iter) = scene_spawner.iter_instance_entities(instance_id) {
                entity_iter.for_each(|entity| {
                    commands.entity(entity).insert(PHAGE_PASS_LAYER);
                    commands.entity(entity).insert(CrownModel);
                });

                *crown_loaded = true;
            }
        }
    }
}

fn phage_rotation(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<PhageWholeModel>>,
) {
    for mut transform in query.iter_mut() {
        let rotation = Quat::from_rotation_y(2.0 * PI * 0.2 * time.delta_seconds());
        transform.rotate(rotation);
    }
}

#[allow(clippy::type_complexity)]
fn selected_phage(
    game_settings: ResMut<ReadWriteGameSettings>,
    mut set: ParamSet<(
        Query<&mut Transform, With<AntennaModel>>,
        Query<&mut Transform, With<CowboyModel>>,
        Query<&mut Transform, With<CrownModel>>,
    )>
) {
    // is antenna showing?
    for mut transform in set.p0().iter_mut() {
        if game_settings.pending_profile.phage == PhageVariant::Antenna {
            transform.translation.y = 0.;
        } else {
            transform.translation.y = 999999.9;
        }
    }

    // is cowboy hat showing?
    for mut transform in set.p1().iter_mut() {
        if game_settings.pending_profile.phage == PhageVariant::Cowboy {
            transform.translation.y = 0.;
        } else {
            transform.translation.y = 999999.9;
        }
    }

    // is crown showing?
    for mut transform in set.p2().iter_mut() {
        if game_settings.pending_profile.phage == PhageVariant::Crown {
            transform.translation.y = 0.;
        } else {
            transform.translation.y = 999999.9;
        }
    }
}

pub struct PhageEngine {
    render_graph_state: RenderGraphState
}

impl Default for PhageEngine {
    fn default() -> Self {
        PhageEngine {
            render_graph_state: Unloaded
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RenderGraphState {
    Unloaded,
    Loaded,
}

#[allow(clippy::unnecessary_unwrap)]
fn load(
    mut query: Query<&mut Transform, With<Phage3dCamera>>,
    mut state: ResMut<PhageEngine>,
){
    println!("[Phage Selector] LLL");

    // render nodes in the correct order
    // this is what lets everything get layered correctly
    // if query.is_some() && state.render_graph_state == Unloaded {
    //     let mut render_graph = render_graph.unwrap();
    //     render_graph.add_node_edge(node::MAIN_PASS_DEPENDENCIES, PHAGE_PASS_DRIVER).unwrap();
    //     render_graph.add_node_edge(node::CLEAR_PASS_DRIVER, PHAGE_PASS_DRIVER).unwrap();
    //     render_graph.add_node_edge(PHAGE_PASS_DRIVER, node::MAIN_PASS_DRIVER).unwrap();
    //
    //     println!("[Phage Selector] added graph edges for phage view");
    //     state.render_graph_state = RenderGraphState::Loaded;
    // }
}

#[allow(clippy::unnecessary_unwrap)]
fn unload(
    mut query: Query<&mut Camera, With<Phage3dCamera>>,
    mut state: ResMut<PhageEngine>,
){
    println!("[Phage Selector] UUU");

    for mut transform in query.iter_mut() {
        println!("lol");
    }

    // if query.is_some() && state.render_graph_state == RenderGraphState::Loaded {
    //     let mut render_graph = query.unwrap();
    //     render_graph.remove_node_edge(node::MAIN_PASS_DEPENDENCIES, PHAGE_PASS_DRIVER).unwrap();
    //     render_graph.remove_node_edge(node::CLEAR_PASS_DRIVER, PHAGE_PASS_DRIVER).unwrap();
    //     render_graph.remove_node_edge(PHAGE_PASS_DRIVER, node::MAIN_PASS_DRIVER).unwrap();
    //
    //     println!("[Phage Selector] removed graph edges for phage view");
    //     state.render_graph_state = Unloaded;
    // }
}