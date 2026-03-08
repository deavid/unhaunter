use bevy::prelude::*;
use bevy_platform::collections::HashMap;

use crate::state::TileState;
use unspatial_core::boardposition::BoardPosition;

/// The `RoomTopology` resource manages room boundaries, mapping each
/// board position to the name of the room it belongs to.
#[derive(Clone, Default, Resource)]
pub struct RoomTopology {
    /// Maps each board position to the name of the room it belongs to.
    pub room_tiles: HashMap<BoardPosition, String>,
}

impl RoomTopology {
    pub fn reset(&mut self) {
        self.room_tiles.clear();
    }
}

/// The `RoomStateMap` resource tracks the current dynamic state of each room.
/// This resource is replicated to ensure all clients have synchronized room states.
#[derive(Clone, Default, Resource, Debug, serde::Serialize, serde::Deserialize)]
pub struct RoomStateMap {
    /// Tracks the current state of each room, using the room name as the key.
    pub room_state: HashMap<String, TileState>,
}

impl RoomStateMap {
    pub fn reset(&mut self) {
        self.room_state.clear();
    }
}
