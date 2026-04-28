use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::events::walkie_types::WalkieEvent;

/// Sent by a client to propose a walkie event to the server.
///
/// The server validates this through its authoritative `WalkiePlay` resource (cooldowns,
/// priority bar), and if accepted, broadcasts `BroadcastWalkieEvent` to all clients.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct ProposeWalkieEvent {
    pub event: WalkieEvent,
}

/// Sent by the server to all clients when a walkie event has been accepted and should play.
///
/// Clients receive this and force-queue the event for local audio playback.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct BroadcastWalkieEvent {
    pub event: WalkieEvent,
    /// Seed for deterministic voice line selection.
    pub seed: u64,
}
