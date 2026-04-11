use crate::events::GhostInteractionType;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use unspatial_core::position::Position;

/// Replicated component placed on an interacted-with entity when a ghost performs
/// an interaction that has an associated sound effect.
///
/// `unghost-presentation` watches `Changed<GhostInteractionSoundCue>` and plays
/// the appropriate sound locally on every client — no broadcast message needed.
#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct GhostInteractionSoundCue {
    /// World position of the interaction (for spatial audio).
    pub position: Position,
    /// Which type of interaction triggered this cue.
    pub kind: GhostInteractionType,
    /// Server elapsed time when the interaction occurred. Used for late-join
    /// staleness checks in the presentation layer.
    pub triggered_at: f64,
}

impl Default for GhostInteractionSoundCue {
    fn default() -> Self {
        Self {
            position: Position::default(),
            kind: GhostInteractionType::Toggle, // sentinel — Toggle has no sound
            triggered_at: 0.0,
        }
    }
}
