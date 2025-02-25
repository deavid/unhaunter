use bevy::prelude::*;

use crate::types::tiledmap::map::MapLayer;

/// Event triggered to load a new level from a TMX map file.
///
/// This event initiates the level loading process, despawning existing entities,
/// loading map data, and spawning new entities based on the TMX file.
#[derive(Debug, Clone, Event)]
pub struct LoadLevelEvent {
    /// The file path to the TMX map file to be loaded.
    pub map_filepath: String,
}

#[derive(Debug, Clone, Event)]
pub struct LevelLoadedEvent {
    /// The file path that has been loaded.
    pub map_filepath: String,
    /// The layers of the map loaded
    pub layers: Vec<(usize, MapLayer)>,
}

#[derive(Debug, Clone, Event, Default)]
pub struct LevelReadyEvent {
    pub open_van: bool,
}
