use std::{fs, path::PathBuf};

use bevy::{prelude::*, utils::HashMap};
use bevy_kira_audio::{Audio, AudioSource, *};
use leafwing_input_manager::prelude::ActionState;
use rand::seq::SliceRandom;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::game::controller::PlayerAction;

pub struct GameAudioPlugin;

struct BgmChannel;
struct SfxChannel;

#[derive(Debug, Display, EnumIter, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Sfx {
    GameStart,
    Generator,
    PhageCombat,
    Replicator,
    UserInterface,
    VectorSlide,
}

#[derive(Deref, DerefMut)]
pub struct PlayRandomSfx(Sfx);

#[derive(Default)]
pub struct SfxLibrary {
    map: HashMap<Sfx, Vec<Handle<AudioSource>>>,
}

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_audio_channel::<BgmChannel>()
            .add_audio_channel::<SfxChannel>()
            .add_event::<PlayRandomSfx>()
            .add_startup_system(play_bgm)
            .add_system(play_sfx)
            .add_system(audio_example_usage)
            .add_startup_system(load_all_sfx);
    }
}

fn audio_example_usage(
    mut sfx_event: EventWriter<PlayRandomSfx>,
    actions: Query<&ActionState<PlayerAction>>,
) {
    let actions = actions.single();

    if actions.just_pressed(PlayerAction::Scream) {
        sfx_event.send(PlayRandomSfx(Sfx::VectorSlide));
    }
}

fn play_sfx(
    mut sfx_event: EventReader<PlayRandomSfx>,
    channel: Res<AudioChannel<SfxChannel>>,
    library: Res<SfxLibrary>,
) {
    for sfx in sfx_event.iter() {
        let sfx_list = library.map.get(sfx).unwrap();
        if let Some(sfx) = sfx_list.choose(&mut rand::thread_rng()) {
            channel.play(sfx.clone());
        }
    }
}

fn load_all_sfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut library = SfxLibrary::default();
    for sfx in Sfx::iter() {
        library.map.insert(sfx, load_sfx(sfx, &asset_server));
    }
    commands.insert_resource(library);
}

fn load_sfx(sfx: Sfx, asset_server: &AssetServer) -> Vec<Handle<AudioSource>> {
    //Yuck but windows/linux
    let mut sfx_path = PathBuf::new();
    sfx_path.push("assets");
    sfx_path.push("audio");
    sfx_path.push("sounds");
    sfx_path.push(sfx.to_string());

    let audio_paths = fs::read_dir(sfx_path).unwrap();

    let mut to_return = Vec::new();
    for path in audio_paths {
        //Yuck but need to remove the assets/
        let path: String = path
            .unwrap()
            .path()
            .display()
            .to_string()
            .chars()
            .skip(7)
            .collect();
        info!("Loading sfx: {}", path);
        to_return.push(asset_server.load(&path));
    }
    to_return
}

fn play_bgm(asset_server: Res<AssetServer>, bgm: Res<AudioChannel<BgmChannel>>) {
    //Yuck but windows/linux
    let mut bgm_path = PathBuf::new();
    bgm_path.push("audio");
    bgm_path.push("music");
    bgm_path.push("GameplayMusicROUGH.wav");
    bgm.play_looped(asset_server.load(bgm_path));
}
