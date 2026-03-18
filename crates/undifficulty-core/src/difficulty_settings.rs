//! Difficulty Settings Trait
//!
//! This trait defines the interface for querying gameplay settings based on difficulty level.
//! The implementation is in the undifficulty-core crate to avoid circular dependencies.

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

    // UI/Gameplay
    fn van_auto_open(&self) -> bool;
    fn difficulty_name(&self) -> &'static str;
    fn difficulty_description(&self) -> &'static str;
    fn difficulty_score_multiplier(&self) -> f32;
}
