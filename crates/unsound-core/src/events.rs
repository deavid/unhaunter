use bevy::prelude::*;

use unspatial_core::position::Position;

/// Represents an event to play a sound effect at a specific location in the game
/// world.
///
/// This event is used to trigger the playback of a sound with volume adjusted
/// based on the distance to the player's position.
#[derive(Debug, Message, Clone)]
pub struct SoundEvent {
    /// The path to the sound file to be played.
    pub sound_file: String,
    /// The initial volume of the sound effect (this will be adjusted based on
    /// distance).
    pub volume: f32,
    /// The position in the game world where the sound is originating from.
    pub position: Option<Position>,
    /// Whether this sound should be broadcasted to other players in multiplayer.
    pub broadcast: bool,
}
