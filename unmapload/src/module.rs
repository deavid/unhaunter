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

pub use crate::level_setup::LoadLevelSystemParam;

use crate::level_finalization;
use crate::level_setup;
use bevy::prelude::App;

pub(crate) fn app_setup(app: &mut App) {
    level_finalization::app_setup(app);
    level_setup::app_setup(app);
}
