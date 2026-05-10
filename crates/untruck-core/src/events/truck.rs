use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use unspatial_core::position::Position;

#[derive(Clone, Debug, Message, PartialEq, Eq)]
pub enum TruckUIEvent {
    EndMission,
    ExitTruck,
    CraftRepellent,
}

/// Authoritative truck-domain audio broadcast for shared-world truck actions.
#[derive(Clone, Debug, Message, Serialize, Deserialize)]
pub struct TruckAudioMessage {
    pub sound_file: String,
    pub volume: f32,
    pub position: Position,
}
