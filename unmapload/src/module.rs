//! # Level Management Module
//!
//! This module handles the loading and management of game levels in Unhaunter, including:
//!
//! * Loading TMX map data and setting up game environments
//! * Spawning Bevy entities for map tiles, players, ghosts, and other game objects
//! * Initializing entity components based on TMX data and game logic
//! * Managing room-related events and states (lighting conditions, interactive behavior)
//! * Handling post-load processing like prebaked lighting and ghost influence assignment
//!
//! The module is organized into several submodules with specialized responsibilities:
//!
//! * `sprite_db` - Handles population of sprite database with tile data
//! * `tile_spawning` - Manages individual tile entity spawning and configuration
//! * `entity_spawning` - Handles player, ghost and ambient entity creation
//! * `level_setup` - Core level initialization and field setup
//! * `level_finalization` - Post-load processing and environment preparation
//! * `influence_system` - Ghost influence assignment to objects

use bevy::prelude::*;
use uncore::events::loadlevel::{LevelLoadedEvent, LevelReadyEvent, LoadLevelEvent};

// Re-export the public items from submodules
pub use crate::level_finalization::after_level_ready;
pub use crate::level_finalization::load_map_add_prebaked_lighting;
pub use crate::level_finalization::process_pre_meshes;
pub use crate::level_setup::LoadLevelSystemParam;
pub use crate::level_setup::load_level_handler;

/// Sets up all the level-related systems and events in the app
///
/// Note: This function is kept for compatibility but is no longer needed
/// when using the UnMapLoadPlugin.
pub fn app_setup(app: &mut App) {
    app.add_event::<LoadLevelEvent>()
        .add_event::<LevelLoadedEvent>()
        .add_event::<LevelReadyEvent>()
        .add_systems(PostUpdate, crate::level_setup::load_level_handler)
        .add_systems(Update, (process_pre_meshes, after_level_ready))
        .add_systems(
            Update,
            load_map_add_prebaked_lighting.run_if(on_event::<LevelReadyEvent>),
        )
        .add_systems(
            Update,
            crate::influence_system::assign_ghost_influence_system,
        );
}
