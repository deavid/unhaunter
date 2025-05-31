use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_persistent::Persistent;
use unprofile::data::PlayerProfileData;
use unsettings::audio::AudioSettings;

use uncore::difficulty::CurrentDifficulty;
use uncore::resources::summary_data::SummaryData;
use uncore::{
    components::board::position::Position, events::sound::SoundEvent,
    resources::board_data::BoardData,
};

/// A collection of resources and commands frequently used by gear-related systems.
#[derive(SystemParam)]
pub struct GearStuff<'w, 's> {
    /// Access to the game's board data, including collision, lighting, and temperature
    /// fields.
    pub bf: ResMut<'w, BoardData>,
    /// Access to summary data, which tracks game progress and statistics.
    pub summary: ResMut<'w, SummaryData>,
    /// Allows gear systems to spawn new entities (e.g., for sound effects).
    pub commands: Commands<'w, 's>,
    /// Provides access to the asset server for loading sound effects.
    pub asset_server: Res<'w, AssetServer>,
    /// Access to the current game time.
    pub time: Res<'w, Time>,
    /// Event writer for sending sound events.
    pub sound_events: EventWriter<'w, SoundEvent>,
    /// Access to the current difficulty.
    pub difficulty: Res<'w, CurrentDifficulty>,
    /// Audio settings from the game.
    pub audio_settings: Res<'w, Persistent<AudioSettings>>,
    /// Player profile data.
    pub player_profile: Res<'w, Persistent<PlayerProfileData>>,
}

impl GearStuff<'_, '_> {
    /// Plays a sound effect using the specified file path and volume from the given
    /// position.
    pub fn play_audio(&mut self, sound_file: String, volume: f32, position: &Position) {
        // Create a SoundEvent with the required data
        let sound_event = SoundEvent {
            sound_file,
            volume,
            position: Some(*position),
        };

        // Send the SoundEvent to be handled by the sound playback system
        self.sound_events.send(sound_event);
    }

    /// Plays a sound effect without having a position volume modifier.
    pub fn play_audio_nopos(&mut self, sound_file: String, volume: f32) {
        // Create a SoundEvent with the required data
        let sound_event = SoundEvent {
            sound_file,
            volume,
            position: None,
        };

        // Send the SoundEvent to be handled by the sound playback system
        self.sound_events.send(sound_event);
    }
}
