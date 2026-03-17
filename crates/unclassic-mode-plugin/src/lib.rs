//! Classic mode game mechanics for Unhaunter.
//!
//! This plugin implements the core gameplay systems for the classic paranormal investigation mode,
//! including ghost hydration, player state management, evidence perception, environmental mechanics
//! (sanity/sound effects), object charging, and mission evaluation. It provides two plugins:
//!
//! - [`UnhaunterClassicModeCorePlugin`]: Core systems for spawning and managing entities
//! - [`ClassicModePlugin`]: Client-side systems for rendering ghosts and UI

pub(crate) mod environmental_mechanics;
pub(crate) mod evidence_perception;
pub(crate) mod game_ui;
pub(crate) mod gear_ui;
pub(crate) mod looking_gear;
pub(crate) mod object_charge;
pub(crate) mod resources;
pub(crate) mod roomchanged;
pub(crate) mod systems;

mod evaluator;
mod influence_system;
mod selection;

pub mod plugin;
