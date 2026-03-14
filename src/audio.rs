use bevy::prelude::*;

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, _app: &mut App) {
        // Audio disabled for now — the MP3 file is corrupted and
        // procedural WAV generation causes decoder panics in Bevy 0.15's
        // Symphonia backend. Will re-enable once a valid .ogg music file
        // is available.
    }
}
