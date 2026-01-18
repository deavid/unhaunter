use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_persistent::Persistent;
use unevents_core::events::sound::SoundEvent;
use unprofile_core::profile::PlayerProfileData;
use unsettings_core::audio::AudioSettings;
use unspatial_core::position::Position;

/// A collection of resources frequently used for audio playback.
#[derive(SystemParam)]
pub struct GearAudio<'w> {
    /// Provides access to the asset server for loading sound effects.
    pub asset_server: Res<'w, AssetServer>,
    /// Access to the current game time.
    pub time: Res<'w, Time>,
    /// Event writer for sending sound events.
    pub sound_events: MessageWriter<'w, SoundEvent>,
    /// Audio settings from the game.
    pub audio_settings: Res<'w, Persistent<AudioSettings>>,
    /// Player profile data.
    pub player_profile: Res<'w, Persistent<PlayerProfileData>>,
}

impl GearAudio<'_> {
    /// Plays a sound effect using the specified file path and volume from the given
    /// position.
    pub fn play_audio(&mut self, sound_file: String, volume: f32, position: &Position) {
        // Add defensive check to prevent empty file paths
        if sound_file.is_empty() {
            warn!("Attempted to play a sound with an empty file path. Ignoring.");
            return;
        }

        // Create a SoundEvent with the required data
        let sound_event = SoundEvent {
            sound_file,
            volume,
            position: Some(*position),
        };

        // Send the sound event
        self.sound_events.write(sound_event);
    }

    /// Plays a sound effect without having a position volume modifier.
    pub fn play_audio_nopos(&mut self, sound_file: String, volume: f32) {
        // Add defensive check to prevent empty file paths
        if sound_file.is_empty() {
            warn!("Attempted to play a sound with an empty file path. Ignoring.");
            return;
        }

        // Create a SoundEvent with the required data
        let sound_event = SoundEvent {
            sound_file,
            volume,
            position: None,
        };

        // Send the SoundEvent to be handled by the sound playback system
        self.sound_events.write(sound_event);
    }
}
