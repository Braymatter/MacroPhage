use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use iyes_progress::{ProgressCounter, ProgressPlugin};

pub struct ModelPlugin;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum LoadingState {
    AssetLoading,
    Loaded,
}

impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        //Loopless states aren't working with the asset loader plugin for some cursed reason...
        //Probably they have weird version compat problems
        app.add_state(LoadingState::AssetLoading);
        AssetLoader::new(LoadingState::AssetLoading)
            //.continue_to_state(LoadingState::Loaded)
            .with_collection::<ModelAssets>()
            .with_collection::<ModelAssets2>()
            .build(app);
        app.add_plugin(
            ProgressPlugin::new(LoadingState::AssetLoading).continue_to(LoadingState::Loaded),
        )
        .add_system(model_loading_progress)
        .add_system_set(SystemSet::on_enter(LoadingState::Loaded).with_system(spawn_gltfs));
    }
}

fn model_loading_progress(progress: Option<Res<ProgressCounter>>) {
    if let Some(_progress) = progress {
        //info!("Progress {:?}", progress.progress());
    }
}

#[derive(AssetCollection)]
pub struct ModelAssets {
    #[asset(path = "3DArt/Avatar Phage/Avatar Phage.glb#Scene0")]
    avatar_phage: Handle<Scene>,
    #[asset(path = "3DArt/Cell/Cell.gltf#Scene0")]
    cell: Handle<Scene>,
    #[asset(path = "3DArt/Cell Variations/Cell Var 1.gltf#Scene0")]
    cell_var_1: Handle<Scene>,
    #[asset(path = "3DArt/Cell Variations/Cell Var 2.gltf#Scene0")]
    cell_var_2: Handle<Scene>,
    //This one is cursed, something wrong with the gltf file
    //#[asset(path = "3DArt/Cell Variations/Cell Var 3.gltf#Scene0")]
    //cell_var_3: Handle<Scene>,
    #[asset(path = "3DArt/Cosmetics Avatar Phage/Antenna.glb#Scene0")]
    antenna: Handle<Scene>,
    #[asset(path = "3DArt/Nexus/Nexus.gltf#Scene0")]
    nexus: Handle<Scene>,
    #[asset(path = "3DArt/Recombinators/Recombinator.gltf#Scene0")]
    recombinator: Handle<Scene>,
    //#[asset(path = "3DArt/Replicator/Replicator.gltf#Scene0")]
    //replicator: Handle<Scene>,
}

#[derive(AssetCollection)]
pub struct ModelAssets2 {
    #[asset(path = "3DArt/Replicator/Replicator.gltf#Scene0")]
    replicator: Handle<Scene>,
}

pub fn spawn_model(commands: &mut Commands, asset: Handle<Scene>, translation: Vec3) {
    commands
        .spawn_bundle(TransformBundle::from_transform(
            Transform::from_scale(Vec3::splat(1.0)).with_translation(translation),
        ))
        .with_children(|commands| {
            commands.spawn_scene(asset.clone());
        });
}

fn spawn_gltfs(mut commands: Commands, assets: Res<ModelAssets>, assets2: Res<ModelAssets2>) {
    info!("Models Loaded");
    spawn_model(&mut commands, assets.cell.clone(), Vec3::new(0.0, 0.0, 0.0));
    spawn_model(
        &mut commands,
        assets2.replicator.clone(),
        Vec3::new(0.0, 0.0, 2.0),
    );
    spawn_model(
        &mut commands,
        assets.recombinator.clone(),
        Vec3::new(0.0, 0.0, 4.0),
    );
    spawn_model(
        &mut commands,
        assets.antenna.clone(),
        Vec3::new(0.0, 0.0, 6.0),
    );
    spawn_model(
        &mut commands,
        assets.nexus.clone(),
        Vec3::new(0.0, 0.0, 8.0),
    );
    spawn_model(
        &mut commands,
        assets.cell_var_1.clone(),
        Vec3::new(0.0, 0.0, 10.0),
    );
    spawn_model(
        &mut commands,
        assets.cell_var_2.clone(),
        Vec3::new(0.0, 0.0, 12.0),
    );
    spawn_model(
        &mut commands,
        assets.avatar_phage.clone(),
        Vec3::new(0.0, -10.0, 20.0),
    );
}
