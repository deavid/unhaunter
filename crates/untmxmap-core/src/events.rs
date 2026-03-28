use bevy::prelude::*;

use unboard_core::types::floor::FloorLevelMapping;
use untiled_core::tiledmap::map::MapLayer;

/// Internal pipeline event carrying raw map data from the Tiled loader to the map setup system.
/// Only consumed within the T3 map pipeline (`untmxmap-plugin` → `unmapload-plugin`).
/// External crates should listen to `unmapload_core::events::loadlevel::LevelLoadedEvent` instead.
#[derive(Debug, Clone, Message)]
pub struct LevelDataEvent {
    /// The file path that has been loaded.
    pub map_filepath: String,
    /// The layers of the map loaded
    pub layers: Vec<(usize, MapLayer)>,
    /// Floor level mapping information
    pub floor_mapping: FloorLevelMapping,
}
