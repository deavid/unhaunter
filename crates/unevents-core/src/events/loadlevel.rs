use bevy::prelude::*;

use unboard_core::types::floor::FloorLevelMapping;
use untiled_core::tiledmap::map::MapLayer;

/// Event triggered to load a new level from a TMX map file.
///
/// This event initiates the level loading process, despawning existing entities,
/// loading map data, and spawning new entities based on the TMX file.
#[derive(Debug, Clone, Message)]
pub struct LoadLevelEvent {
    /// The file path to the TMX map file to be loaded.
    pub map_filepath: String,
}

#[derive(Debug, Clone, Message)]
pub struct LevelLoadedEvent {
    /// The file path that has been loaded.
    pub map_filepath: String,
    /// The layers of the map loaded
    pub layers: Vec<(usize, MapLayer)>,
    /// Floor level mapping information
    pub floor_mapping: FloorLevelMapping,
}

#[derive(Debug, Clone, Message, Default)]
pub struct LevelReadyEvent {
    pub open_van: bool,
}

#[derive(Debug, Clone, Message)]
pub struct MapGeometryInitializedEvent {
    pub map_size: (usize, usize, usize),
    pub origin: (i32, i32, i32),
}

#[derive(Debug, Clone, Message, Default)]
pub struct MapEntitiesReadyEvent {
    pub movable_objects: Vec<Entity>,
    pub player_spawn_points: Vec<unspatial_core::position::Position>,
    pub hostile_spawn_points: Vec<unspatial_core::position::Position>,
    pub van_entry_points: Vec<unspatial_core::position::Position>,
}
