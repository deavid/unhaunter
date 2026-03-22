use crate::events::SoundEvent;
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

impl AudioEmitter<'_> {
    /// Plays a sound effect using the specified file path and volume from the given
    /// position.
    pub fn play_audio(&mut self, sound_file: String, volume: f32, position: &Position) {
        self.play_audio_ex(sound_file, volume, Some(*position), true);
    }

    /// Plays a sound effect without having a position volume modifier.
    pub fn play_audio_nopos(&mut self, sound_file: String, volume: f32) {
        self.play_audio_ex(sound_file, volume, None, true);
    }

    /// Plays a sound effect using the specified file path and volume from the given
    /// position, but does not broadcast it to other players in multiplayer.
    pub fn play_audio_local(&mut self, sound_file: String, volume: f32, position: &Position) {
        self.play_audio_ex(sound_file, volume, Some(*position), false);
    }

    /// Plays a sound effect without having a position volume modifier and without
    /// broadcasting it to other players in multiplayer.
    pub fn play_audio_nopos_local(&mut self, sound_file: String, volume: f32) {
        self.play_audio_ex(sound_file, volume, None, false);
    }

    fn play_audio_ex(
        &mut self,
        sound_file: String,
        volume: f32,
        position: Option<Position>,
        broadcast: bool,
    ) {
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
            broadcast,
        };

        // Send the sound event
        self.sound_events.write(sound_event);
    }
}
