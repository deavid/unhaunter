use bevy::prelude::*;
use bevy_platform::collections::HashMap;

use crate::{behavior::TileState, components::board::boardposition::BoardPosition};

/// The `RoomDB` resource manages room-related data, including room boundaries and
/// states.
#[derive(Clone, Default, Resource)]
pub struct RoomDB {
    /// Maps each board position to the name of the room it belongs to. This defines
    /// the boundaries of each room in the game world.
    pub room_tiles: HashMap<BoardPosition, String>,
    /// Tracks the current state of each room, using the room name as the key. The
    /// exact nature of the room state is not explicitly defined but could include
    /// things like:
    ///
    /// * Lighting conditions (lit/unlit).
    ///
    /// * Presence of specific objects or entities.
    ///
    /// * Temperature or other environmental factors.
    pub room_state: HashMap<String, TileState>,
}
