use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_mod_picking::{PickableBundle, PickingEvent};
use iyes_progress::{ProgressCounter, ProgressPlugin};

pub struct ModelPlugin;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum LoadingStates {
    AssetLoading,
    Loaded,
}

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        //Loopless states aren't working with the asset loader plugin for some cursed reason...
        //Probably they have weird version compat problems
        app.add_state(LoadingStates::AssetLoading)
            //.add_loading_state(
            //LoadingState::new(LoadingStates::AssetLoading)
            //.with_collection::<ModelAssets>()
            //.with_collection::<NodeTenentAssets>(),
            //)
            //.add_plugin(
            //ProgressPlugin::new(LoadingStates::AssetLoading).continue_to(LoadingStates::Loaded),
            //)
            //.add_system(model_loading_progress)
            .add_system(model_coloring)
            .add_startup_system(load_gltfs);
    }
}

fn model_loading_progress(progress: Option<Res<ProgressCounter>>) {
    if let Some(ref progress) = progress {
        info!("Progress {:?}", progress.progress());
    }
}

#[derive(AssetCollection)]
pub struct ModelAssets {
    #[asset(path = "3DArt/Avatar Phage/Avatar Phage.glb#Scene0")]
    avatar_phage: Handle<Scene>,
    #[asset(path = "3DArt/Cosmetics Avatar Phage/Antenna.glb#Scene0")]
    antenna: Handle<Scene>,
    #[asset(path = "3DArt/Recombinators/Recombinator.gltf#Scene0")]
    recombinator: Handle<Scene>,
}

#[derive(AssetCollection)]
pub struct NodeTenentAssets {
    #[asset(path = "3DArt/Replicator/Replicator.gltf#Scene0")]
    pub replicator: Handle<Scene>,
    #[asset(path = "3DArt/Nexus/Nexus.gltf#Scene0")]
    pub nexus: Handle<Scene>,
    #[asset(path = "3DArt/Cell/Cell.gltf#Scene0")]
    pub cell: Handle<Scene>,
    #[asset(path = "3DArt/Cell Variations/Cell Var 1.gltf#Scene0")]
    pub cell_var_1: Handle<Scene>,
    #[asset(path = "3DArt/Cell Variations/Cell Var 2.gltf#Scene0")]
    pub cell_var_2: Handle<Scene>,
    //This one is cursed, something wrong with the gltf file
    //#[asset(path = "3DArt/Cell Variations/Cell Var 3.gltf#Scene0")]
    //cell_var_3: Handle<Scene>,
}

pub fn model_coloring(
    nodes: Query<(&crate::game::Node, &Parent)>,
    parents: Query<&Children>,
    mut mat_handles: Query<&mut Handle<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (node, parent) in &nodes {
        let color = node.force.color();
        //If I am your parent then I must have children
        let children = parents.get(parent.get()).unwrap();
        //Crawl through children until you find one with the material
        let mut children: Vec<&Entity> = children.iter().collect();
        while !children.is_empty() {
            //Children is not empty
            let checking = children.pop().unwrap();
            if let Ok(mut material) = mat_handles.get_mut(*checking) {
                // This is the child
                let mut mat = materials.get(&material).unwrap().clone();
                mat.emissive = color;
                *material = materials.add(mat);
            }
            //Add my children to the list to search
            if let Ok(my_children) = parents.get(*checking) {
                let mut my_children: Vec<&Entity> = my_children.iter().collect();
                children.append(&mut my_children);
            }
        }
    }
}

pub fn spawn_model(
    commands: &mut Commands,
    asset: Handle<Scene>,
    meshes: &mut Assets<Mesh>,
    node: &crate::game::Node,
) -> Entity {
    commands
        .spawn_bundle(SceneBundle {
            scene: asset,
            transform: Transform::from_translation(node.position),
            ..default()
        })
        .with_children(|commands| {
            commands
                .spawn_bundle(SpatialBundle::default())
                .insert(meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.7,
                    subdivisions: 4,
                })))
                .insert((*node).clone())
                .insert(Name::new("Clickable"))
                .insert_bundle(PickableBundle::default());
        })
        .id()
}

fn load_gltfs(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Models Loading");
    commands.insert_resource(ModelAssets {
        avatar_phage: asset_server.load("3DArt/Avatar Phage/Avatar Phage.glb#Scene0"),
        antenna: asset_server.load("3DArt/Cosmetics Avatar Phage/Antenna.glb#Scene0"),
        recombinator: asset_server.load("3DArt/Recombinators/Recombinator.gltf#Scene0"),
    });
    commands.insert_resource(NodeTenentAssets {
        replicator: asset_server.load("3DArt/Replicator/Replicator.gltf#Scene0"),
        nexus: asset_server.load("3DArt/Nexus/Nexus.gltf#Scene0"),
        cell: asset_server.load("3DArt/Cell/Cell.glb#Scene0"),
        cell_var_1: asset_server.load("3DArt/Cell Variations/Cell Var 1.gltf#Scene0"),
        cell_var_2: asset_server.load("3DArt/Cell Variations/Cell Var 2.gltf#Scene0"),
    });
}
