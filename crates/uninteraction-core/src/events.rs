use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use unspatial_core::position::Position;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum InteractionExecutionType {
    ChangeState,
    ReadRoomState,
}

/// Event triggered when the player enters a new room or when a significant
/// room-related change occurs.
///
/// This event is used to trigger actions like opening the van UI or updating the
/// state of interactive objects based on the room's current state.
#[derive(Clone, Debug, Default, Message)]
pub struct RoomChangedEvent {
    /// Set to `true` if the event is triggered during level initialization.
    pub initialize: bool,
    /// Set to `true` if the van UI should be opened automatically (e.g., when the
    /// player returns to the starting area).
    pub open_van: bool,
}

/// Event triggered to synchronize all interactive entities with the `RoomStateMap`.
///
/// This is typically fired after an interaction changes a room state, or
/// during level initialization.
#[derive(Clone, Debug, Default, Message)]
pub struct RoomStateSyncEvent;

/// Message sent by a client to request an interactive-object state change.
///
/// Uses board-space integer coordinates to identify the target entity in a
/// map-stable way (all clients load the same map from the same seed).
/// The server finds the entity at `position`, validates the request, and fires
/// `ExecuteInteractionEvent` locally.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct InteractionRequestMessage {
    /// Board-space position (`[x, y, z]`) of the interactive entity.
    pub position: [i32; 3],
    /// Whether to change state or only read the current room state.
    pub ietype: InteractionExecutionType,
    /// If `Some`, force the interaction to transition to this specific tile UID.
    pub force_tuid: Option<u32>,
}

/// Sent by the authoritative interaction domain when an environmental interaction
/// should produce a sound effect on all players.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct PlayInteractionAudioMessage {
    pub sound_file: String,
    pub volume: f32,
    pub position: Position,
}

impl RoomChangedEvent {
    /// Creates a new `RoomChangedEvent` specifically for level initialization.
    ///
    /// The `initialize` flag is set to `true`, and the `open_van` flag is set based on
    /// the given value.
    pub fn init(open_van: bool) -> Self {
        Self {
            initialize: true,
            open_van,
        }
    }
}
