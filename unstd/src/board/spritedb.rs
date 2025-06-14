use super::tiledata::MapTileComponents;
use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use uncore::behavior::SpriteCVOKey;

/// The `SpriteDB` resource stores a database of pre-built Bevy components and
/// sprites for map tiles.
///
/// This resource optimizes map loading and manipulation by:
///
/// * Pre-building Bevy components for each tile type, avoiding redundant entity
///   creation during map loading.
///
/// * Providing efficient lookup of tile components based on their unique identifiers.
///
/// * Indexing tiles based on their visual characteristics (class, variant,
///   orientation) for quick access during interaction events.
#[derive(Clone, Default, Resource)]
pub struct SpriteDB {
    /// Maps a unique tile identifier (tileset name + tile UID) to its pre-built Bevy
    /// components, including the `Bdl` (bundle) and the `Behavior`. This enables
    /// efficient lookup of components during map loading and interaction events.
    pub map_tile: HashMap<(String, u32), MapTileComponents>,
    /// Indexes tile identifiers based on their visual characteristics:
    ///
    /// * `class`: The type of tile (e.g., "Door", "Wall").
    ///
    /// * `variant`:  A specific variation of the tile type (e.g., "wooden", "brick").
    ///
    /// * `orientation`: The direction the tile is facing (e.g., "XAxis", "YAxis").
    ///
    /// This index allows for quick retrieval of tiles that share the same sprite,
    /// which is useful when updating the state of interactive objects that have
    /// multiple instances in the map. For example, when the player opens a door, all
    /// other doors of the same type can be updated efficiently.
    pub cvo_idx: HashMap<SpriteCVOKey, Vec<(String, u32)>>,
}

impl SpriteDB {
    pub fn clear(&mut self) {
        self.map_tile.clear();
        self.cvo_idx.clear();
    }
}
