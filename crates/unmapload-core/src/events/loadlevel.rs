use bevy::prelude::*;

/// Event triggered to load a new level from a TMX map file.
///
/// This event initiates the level loading process, despawning existing entities,
/// loading map data, and spawning new entities based on the TMX file.
#[derive(Debug, Clone, Message)]
pub struct LoadLevelEvent {
    /// The file path to the TMX map file to be loaded.
    pub map_filepath: String,
}

/// Fired by `unmapload-plugin` after level geometry and all tile entities are spawned.
/// This is the public signal that a map load has started; listen to this to initialize
/// domain state in response to a new mission map.
#[derive(Debug, Clone, Message, Default)]
pub struct LevelLoadedEvent;

#[derive(Debug, Clone, Message, Default)]
pub struct MapEntitiesReadyEvent {}
