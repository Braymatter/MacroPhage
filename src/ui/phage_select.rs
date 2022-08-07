use std::f32::consts::PI;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::render::camera::{RenderTarget};
use bevy::render::render_graph::{RenderGraph, RenderGraphContext, SlotValue};
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::{RenderLayers};
use bevy::scene::InstanceId;
use iyes_loopless::prelude::*;
use crate::game::settings::{PhageVariant, ReadWriteGameSettings};
use crate::ui::show_profile_screen;

pub struct PhageSelectPlugin;

#[derive(Component, Default)]
pub struct Phage3dCamera;

pub struct RenderedPhage {
    pub image: Handle<Image>,
}

const PHAGE_PASS_LAYER: RenderLayers = RenderLayers::layer(1);

/// Helps render a phage in the phage selection window.
/// We need to use a different camera to render to a texture.
impl Plugin for PhageSelectPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SceneInstance>()
            .init_resource::<PhageEngine>()
            .add_startup_system(setup)
            .add_system(phage_rotation.run_if(show_profile_screen))
            .add_system(selected_phage.run_if(show_profile_screen))
            .add_system(load.run_if(show_profile_screen))
            .add_system(unload.run_if_not(show_profile_screen))
            .add_system(tag_scene);
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

    commands.spawn_bundle(Camera3dBundle {
        camera_3d: Camera3d {
            //clear_color: ClearColorConfig::Custom(Color::rgba(0., 0., 0., 0.)),
            clear_color: ClearColorConfig::Default,
            ..default()
        },
        camera: Camera {
            priority: -1,
            target: render_target,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(19.0, 27.0, 25.0))
            .looking_at(Vec3::default(), Vec3::Y),
            ..default()
        })
        .insert(PHAGE_PASS_LAYER)
        .insert(Phage3dCamera);

    let phage_instance_id = scene_spawner.spawn(asset_server.load("meshes/Avatar Phage.glb#Scene0"));
    let antenna_instance_id = scene_spawner.spawn(asset_server.load("meshes/Antenna.glb#Scene0"));
    let cowboy_instance_id = scene_spawner.spawn(asset_server.load("meshes/CowboyHat.glb#Scene0"));
    let crown_instance_id = scene_spawner.spawn(asset_server.load("meshes/crown.glb#Scene0"));

    scene_instance.phage = Some(phage_instance_id);
    scene_instance.antenna = Some(antenna_instance_id);
    scene_instance.cowboy = Some(cowboy_instance_id);
    scene_instance.crown = Some(crown_instance_id);


    // TODO: file bug with Bevy 0.8, this should work but something is wrong in the view hierarchy introduced with
    //       0.8 with the bundles + layers and simplified pipeline. Old commit works fine on 0.7 to compare.

    // commands.spawn_bundle(TransformBundle::from(Transform::from_xyz(0., 0., 0.)))
    //     .insert(PHAGE_PASS_LAYER)
    //     .insert(PhageWholeModel)
    //     .with_children(|p| {
    //         let phage_instance_id = scene_spawner.spawn_as_child(asset_server.load("meshes/Avatar Phage.glb#Scene0"), p.parent_entity());
    //         let antenna_instance_id = scene_spawner.spawn_as_child(asset_server.load("meshes/Antenna.glb#Scene0"), p.parent_entity());
    //         let cowboy_instance_id = scene_spawner.spawn_as_child(asset_server.load("meshes/CowboyHat.glb#Scene0"), p.parent_entity());
    //         let crown_instance_id = scene_spawner.spawn_as_child(asset_server.load("meshes/crown.glb#Scene0"), p.parent_entity());
    //
    //         scene_instance.phage = Some(phage_instance_id);
    //         scene_instance.antenna = Some(antenna_instance_id);
    //         scene_instance.cowboy = Some(cowboy_instance_id);
    //         scene_instance.crown = Some(crown_instance_id);
    //     });

    // Warning: lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
    commands.spawn_bundle(PointLightBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 10.0, 10.0)),
            ..Default::default()
        })
        .insert(PHAGE_PASS_LAYER);
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
                    println!("[Phage Select] base model ready {}", entity.id());
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
                    println!("[Phage Select] antenna model ready {}", entity.id());
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
                    println!("[Phage Select] cowboy hat model ready {}", entity.id());
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
                    println!("[Phage Select] crown model ready {}", entity.id());
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
    mut query: Query<&mut Transform, With<Phage3dCamera>>,
) {
    for mut transform in query.iter_mut() {
        let rotation = Quat::from_rotation_y(2.0 * PI * 0.2 * time.delta_seconds());
        transform.rotate_around(Vec3::default(), rotation);
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
            render_graph_state: RenderGraphState::Unloaded
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum RenderGraphState {
    Unloaded,
    Loaded,
}

fn load(
    mut query: Query<&mut Camera, With<Phage3dCamera>>,
    mut state: ResMut<PhageEngine>,
){
    if state.render_graph_state == RenderGraphState::Unloaded {
        for mut camera in query.iter_mut() {
            camera.is_active = true;
        }

        println!("[Phage Selector] phage view active");
        state.render_graph_state = RenderGraphState::Loaded;
    }
}

fn unload(
    mut query: Query<&mut Camera, With<Phage3dCamera>>,
    mut state: ResMut<PhageEngine>,
){
    if state.render_graph_state == RenderGraphState::Loaded {
        for mut camera in query.iter_mut() {
            camera.is_active = false;
        }

        println!("[Phage Selector] phage view inactive");
        state.render_graph_state = RenderGraphState::Unloaded;
    }
}
