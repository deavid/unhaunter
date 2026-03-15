use bevy::prelude::*;
use bevy_platform::collections::HashMap;

use crate::state::TileState;
use unspatial_core::boardposition::BoardPosition;

/// Maps each board position to the room name it belongs to.
///
/// All nodes (server and client) maintain this. It is populated during
/// `HydrationStage<2>` in `unrender-plugin` and reset on `OnExit(AppState::InGame)`.
#[derive(Clone, Default, Resource)]
pub struct RoomTopology {
    pub room_tiles: HashMap<BoardPosition, String>,
}

impl RoomTopology {
    pub fn reset(&mut self) {
        self.room_tiles.clear();
    }
}

/// Tracks the current TileState of each named room (e.g. On/Off for lights).
///
/// Written exclusively on the Authority node (server or offline host).
/// All nodes hold this resource (initialised to TileState::Off per room during
/// `HydrationStage<2>`), but only the Authority mutates it after initialisation.
/// Pure clients read this map while authoritative map tile `Behavior` changes are
/// received through component replication.
#[derive(Clone, Default, Resource)]
pub struct RoomStateMap {
    pub room_state: HashMap<String, TileState>,
}

impl RoomStateMap {
    pub fn reset(&mut self) {
        self.room_state.clear();
    }
}
