use crate::events::{LocalSoundEvent, SoundEvent};
use bevy::{ecs::system::SystemParam, prelude::*};
use unspatial_core::position::Position;

/// A collection of resources frequently used for audio playback.
#[derive(SystemParam)]
pub struct AudioEmitter<'w> {
    /// Provides access to the asset server for loading sound effects.
    pub asset_server: Res<'w, AssetServer>,
    /// Access to the current game time.
    pub time: Res<'w, Time>,
    /// Event writer for sending sound events.
    pub sound_events: MessageWriter<'w, SoundEvent>,
}

/// Emits local-only playback requests without network replication.
#[derive(SystemParam)]
pub struct LocalAudioEmitter<'w> {
    /// Provides access to the asset server for loading sound effects.
    pub asset_server: Res<'w, AssetServer>,
    /// Access to the current game time.
    pub time: Res<'w, Time>,
    /// Message writer for local-only sound events.
    pub sound_events: MessageWriter<'w, LocalSoundEvent>,
}

impl AudioEmitter<'_> {
    /// Plays a sound effect using the specified file path and volume from the given
    /// position.
    pub fn play_audio(&mut self, sound_file: String, volume: f32, position: &Position) {
        self.play_audio_ex(sound_file, volume, Some(*position));
    }

    /// Plays a sound effect without having a position volume modifier.
    pub fn play_audio_nopos(&mut self, sound_file: String, volume: f32) {
        self.play_audio_ex(sound_file, volume, None);
    }

    fn play_audio_ex(&mut self, sound_file: String, volume: f32, position: Option<Position>) {
        // Add defensive check to prevent empty file paths
        if sound_file.is_empty() {
            warn!("Attempted to play a sound with an empty file path. Ignoring.");
            return;
        }

        // Create a SoundEvent with the required data
        let sound_event = SoundEvent {
            sound_file,
            volume,
            position,
        };

        // Send the sound event
        self.sound_events.write(sound_event);
    }
}

impl LocalAudioEmitter<'_> {
    /// Plays a sound effect using the specified file path and volume from the given
    /// position locally on this node only.
    pub fn play_audio(&mut self, sound_file: String, volume: f32, position: &Position) {
        self.play_audio_ex(sound_file, volume, Some(*position));
    }

    /// Plays a sound effect locally without a position volume modifier.
    pub fn play_audio_nopos(&mut self, sound_file: String, volume: f32) {
        self.play_audio_ex(sound_file, volume, None);
    }

    fn play_audio_ex(&mut self, sound_file: String, volume: f32, position: Option<Position>) {
        if sound_file.is_empty() {
            warn!("Attempted to play a local sound with an empty file path. Ignoring.");
            return;
        }

        self.sound_events.write(LocalSoundEvent {
            sound_file,
            volume,
            position,
        });
    }
}
