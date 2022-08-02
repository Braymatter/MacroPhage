use bevy::prelude::*;
use bevy_kira_audio::{*, Audio};

pub struct GameAudioPlugin;

#[derive(Component, Default, Clone)]
struct BgmChannel;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin).add_audio_channel::<BgmChannel>().add_startup_system(play_bgm);
    }
}

fn play_bgm(asset_server: Res<AssetServer>, audio: Res<Audio>) {
     audio.play_looped(asset_server.load("audio/music/GameplayMusicROUGH.wav"));
}