use bevy::prelude::*;
use bevy::ecs::entity::{EntityMapper, MapEntities};
use serde::{Deserialize, Serialize};
use unevents_core::events::roomchanged::InteractionExecutionType;
use unghost_core::types::evidence::Evidence;
use unghost_core::types::ghost::types::GhostType;

use crate::network_id::NetworkId;
use unfoundation_core::types::gear::Hand;
use ungear_core::types::gear::kind::GearKind;

/// Sent by the (room-owner) client to request a map change.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct RequestSelectMap {
    pub map_filepath: String,
}

/// Sent by the (room-owner) client to request a difficulty change.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct RequestSelectDifficulty {
    pub difficulty_id: String,
}

/// Sent by the (room-owner) client to start the mission.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct RequestStartMission {
    pub map_seed: u64,
}

// ---------------------------------------------------------------------------
// Interactions
// ---------------------------------------------------------------------------

/// Sent by a client to request an interactive-object state change.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct InteractionRequestMessage {
    /// Board-space position ([x, y, z]) of the interactive entity.
    pub position: [i32; 3],
    /// Whether to change state or only read the current room state.
    pub ietype: InteractionExecutionType,
    /// If Some, force the interaction to transition to this specific tile UID.
    pub force_tuid: Option<u32>,
}

/// Broadcast by the server to all join clients (excluding the originator) to
/// notify them of a remote player's interaction with an interactive map object.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct RemoteInteractionBroadcast {
    pub position: [i32; 3],
    pub ietype: InteractionExecutionType,
    pub force_tuid: Option<u32>,
}

/// One loadout action a join client can request from the server during the truck phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TruckLoadoutAction {
    AddGear(GearKind),
    ClearHand(Hand),
    ClearInventorySlot(usize),
}

/// Sent by a join client to request a loadout change during the truck phase.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct TruckLoadoutMessage {
    pub action: TruckLoadoutAction,
}

/// Local-only Bevy event fired by player_interaction_system on the authority.
#[derive(Debug, Clone, Message)]
pub struct HostInteractionOccurred {
    pub position: [i32; 3],
    pub ietype: InteractionExecutionType,
    pub force_tuid: Option<u32>,
}

/// Local-only Bevy event fired by watch_tween_insertions on the authority.
#[derive(Debug, Clone, Message)]
pub struct HostMovableMotionEvent {
    pub map_bpos: [i32; 3],
    pub start: [f32; 4],
    pub end: [f32; 4],
    pub duration: f32,
    pub ease: u8,
}

/// Broadcast by the server to all join clients when a ghost interaction moves a map object.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct MovableMotionBroadcast {
    pub map_bpos: [i32; 3],
    pub start: [f32; 4],
    pub end: [f32; 4],
    pub duration: f32,
    pub ease: u8,
}

/// Local-only Bevy event fired when a player drops a gear item.
#[derive(Debug, Clone, Message)]
pub struct HostFloorGearDroppedEvent {
    pub kind: GearKind,
    pub pos: [f32; 3],
    pub direction: [f32; 3],
}

/// Local-only Bevy event fired when a player picks up a floor gear item.
#[derive(Debug, Clone, Message)]
pub struct HostFloorGearPickedUpEvent {
    pub pos: [f32; 3],
}

/// Broadcast by the server when a gear item is placed on the floor.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct FloorGearSpawnBroadcast {
    pub kind: GearKind,
    pub pos: [f32; 3],
    pub direction: [f32; 3],
}

/// Broadcast by the server when a floor gear item is picked up.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct FloorGearDespawnBroadcast {
    pub pos: [f32; 3],
}

// ---------------------------------------------------------------------------
// Phase 4: Ghost, Evidence, and Mission messages
// ---------------------------------------------------------------------------

/// Broadcast by the server to all clients to spawn a visual particle effect.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct SpawnParticleNetEvent {
    pub particle_type: String,
    pub position: [f32; 3],
}

/// Sent by a client to toggle evidence in the shared journal.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct RequestJournalEvidenceToggle {
    pub evidence: Evidence,
    pub mark_as_found: bool,
}

/// Sent by a client to update the ghost-type guess in the shared journal.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct RequestJournalGhostToggle {
    pub ghost_type: Option<GhostType>,
}

/// Local event fired when a player dies (server-authoritative).
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct PlayerDiedEvent {
    pub id: NetworkId,
}
