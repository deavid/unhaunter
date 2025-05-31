use bevy::{prelude::*, utils::HashMap};

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

/// Mapping between floor numbers and z-coordinates
#[derive(Debug, Clone)]
pub struct FloorLevelMapping {
    /// Maps floor numbers to z-coordinates (e.g., -1 -> 0, 0 -> 1, 2 -> 2)
    pub floor_to_z: HashMap<i32, usize>,
    /// Maps z-coordinates back to floor numbers
    pub z_to_floor: HashMap<usize, i32>,
    /// Display names for each floor
    pub floor_display_names: HashMap<i32, String>,
    /// Required number of ghost attracting objects for each floor
    pub ghost_attracting_objects: HashMap<i32, i32>,
    /// Required number of ghost repelling objects for each floor
    pub ghost_repelling_objects: HashMap<i32, i32>,
}

#[derive(Debug, Clone, Event)]
pub struct LevelLoadedEvent {
    /// The file path that has been loaded.
    pub map_filepath: String,
    /// The layers of the map loaded
    pub layers: Vec<(usize, MapLayer)>,
    /// Floor level mapping information
    pub floor_mapping: FloorLevelMapping,
}

#[derive(Debug, Clone, Event, Default)]
pub struct LevelReadyEvent {
    pub open_van: bool,
}
