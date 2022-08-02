use std::fs;

use bevy::{prelude::*, utils::HashMap};
use bevy_kira_audio::{Audio, AudioSource, *};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

pub struct GameAudioPlugin;

#[derive(Component, Default, Clone)]
struct BgmChannel;

#[derive(Debug, Display, EnumIter, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Sfx {
    GameStart,
    Generator,
    PhageCombat,
    Replicator,
    UserInterface,
    VectorSlide,
}

#[derive(Default)]
pub struct SfxLibrary {
    map: HashMap<Sfx, Vec<Handle<AudioSource>>>,
}

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_audio_channel::<BgmChannel>()
            .add_startup_system(play_bgm)
            .add_startup_system(load_all_sfx);
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
    let sfx_path = format!("audio/sounds/{}", &sfx.to_string());
    let audio_paths = fs::read_dir(format!("assets/{}", sfx_path)).unwrap();

    let mut to_return = Vec::new();
    for path in audio_paths {
        //Yuck but need to remove the assets/
        let path: String = path.unwrap().path().display().to_string().chars().skip(7).collect();
        info!("Loading sfx: {}", path);
        to_return.push(asset_server.load(&path));
    }
    to_return
}

fn play_bgm(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play_looped(asset_server.load("audio/music/GameplayMusicROUGH.wav"));
}
