//! Difficulty Settings Trait
//!
//! This trait defines the interface for querying gameplay settings based on difficulty level.
//! The implementation is in the undifficulty-core crate to avoid circular dependencies.

use crate::manual_types::ManualChapterIndex;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use unfoundation_core::types::truck::TabContents;
use ungear_core::types::gear::{GearKind, PlayerGearKind};
use unghost_core::types::ghost::definitions::GhostSet;
use untypes_core::difficulty::Difficulty;

/// Trait for querying difficulty-specific game settings
pub trait DifficultySettings {
    // Ghost behavior
    fn ghost_speed(&self) -> f32;
    fn ghost_rage_likelihood(&self) -> f32;
    fn ghost_hunting_aggression(&self) -> f32;
    fn ghost_interaction_frequency(&self) -> f32;
    fn ghost_hunt_duration(&self) -> f32;
    fn ghost_hunt_cooldown(&self) -> f32;
    fn ghost_attraction_to_breach(&self) -> f32;
    fn hunt_provocation_radius(&self) -> f32;
    fn attractive_removal_anger_rate(&self) -> f32;

    // Environment
    fn ambient_temperature(&self) -> f32;
    fn temperature_spread_speed(&self) -> f32;
    fn light_heat(&self) -> f32;
    fn darkness_intensity(&self) -> f32;
    fn environment_gamma(&self) -> f32;

    // Player
    fn max_recoverable_sanity(&self) -> f32;
    fn sanity_drain_rate(&self) -> f32;
    fn health_drain_rate(&self) -> f32;
    fn health_recovery_rate(&self) -> f32;
    fn player_speed(&self) -> f32;

    // Equipment
    fn evidence_visibility(&self) -> f32;
    fn equipment_sensitivity(&self) -> f32;
    fn repellent_craft_limit(&self) -> u32;
    fn truck_gear(&self) -> Vec<GearKind>;
    fn player_gear(&self) -> PlayerGearKind;

    // UI/Gameplay
    fn van_auto_open(&self) -> bool;
    fn default_van_tab(&self) -> TabContents;
    fn difficulty_name(&self) -> &'static str;
    fn difficulty_description(&self) -> &'static str;
    fn difficulty_score_multiplier(&self) -> f32;
    fn tutorial_chapter(&self) -> Option<ManualChapterIndex>;

    // Ghost types
    fn ghost_set(&self) -> GhostSet;

    // Struct assembly
    fn as_struct(&self) -> DifficultyStruct;
}

/// A struct that holds all difficulty settings
#[derive(Debug, Clone, Serialize, Deserialize, Resource, PartialEq)]
pub struct DifficultyStruct {
    pub ghost_speed: f32,
    pub ghost_rage_likelihood: f32,
    pub ghost_hunting_aggression: f32,
    pub ghost_interaction_frequency: f32,
    pub ghost_hunt_duration: f32,
    pub ghost_hunt_cooldown: f32,
    pub ghost_attraction_to_breach: f32,
    pub hunt_provocation_radius: f32,
    pub attractive_removal_anger_rate: f32,
    pub ambient_temperature: f32,
    pub temperature_spread_speed: f32,
    pub light_heat: f32,
    pub darkness_intensity: f32,
    pub environment_gamma: f32,
    pub max_recoverable_sanity: f32,
    pub sanity_drain_rate: f32,
    pub health_drain_rate: f32,
    pub health_recovery_rate: f32,
    pub player_speed: f32,
    pub evidence_visibility: f32,
    pub equipment_sensitivity: f32,
    pub van_auto_open: bool,
    pub default_van_tab: TabContents,
    pub repellent_craft_limit: u32,
    pub player_gear: PlayerGearKind,
    pub ghost_set: GhostSet,
    pub difficulty: Difficulty,
    pub difficulty_name: String,
    pub difficulty_description: String,
    pub difficulty_score_multiplier: f32,
    pub tutorial_chapter: Option<ManualChapterIndex>,
    pub truck_gear: Vec<GearKind>,
}

impl Default for DifficultyStruct {
    fn default() -> Self {
        Difficulty::default().as_struct()
    }
}
