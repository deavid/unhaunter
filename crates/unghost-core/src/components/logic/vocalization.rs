use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use unspatial_core::position::Position;

/// Replicated component on the ghost entity representing its most recent vocalization.
///
/// The authority writes this whenever a roar fires. `unghost-presentation` watches
/// `Changed<GhostVocalization>` and plays the sound locally — no broadcast needed.
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct GhostVocalization {
    /// Path to the sound asset chosen by the authority (e.g. "sounds/ghost-roar-1.ogg").
    /// Empty string means "never vocalised" (default state).
    pub sound_file: String,
    /// Volume for the sound.
    pub volume: f32,
    /// World position of the ghost at the time of vocalization.
    pub position: Position,
    /// Server elapsed time when the vocalization was triggered. Used for late-join
    /// staleness checks in the presentation layer.
    pub triggered_at: f64,
}

impl Default for GhostVocalization {
    fn default() -> Self {
        Self {
            sound_file: String::new(),
            volume: 0.0,
            position: Position::default(),
            triggered_at: 0.0,
        }
    }
}
