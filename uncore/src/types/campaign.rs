use crate::difficulty::Difficulty;
use bevy::prelude::Resource; // If CampaignMissions will be a Resource
use serde::{Deserialize, Serialize}; // If you plan to serialize this later (optional for now) // Assuming Difficulty is in uncore::difficulty

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)] // Add Serialize/Deserialize if you might save/load this directly later
pub struct CampaignMissionData {
    /// Unique identifier for the mission, could be derived from file path or an explicit ID.
    /// For now, let's use the map file path as a simple unique ID.
    pub id: String, // Typically the map_filepath

    /// The full file path to the .tmx map file.
    pub map_filepath: String,

    /// Human-readable name of the mission (from TMX `display_name`).
    pub display_name: String,

    /// Flavor text or briefing for the mission (from TMX `flavor_text`).
    pub flavor_text: String,

    /// Sorting order string (e.g., "01A", "02B") (from TMX `campaign_order`).
    pub order: String,

    /// The fixed difficulty for this campaign mission (parsed from TMX `campaign_difficulty`).
    pub difficulty: Difficulty,
    // Note: `is_campaign_mission` is used for filtering during loading, not usually stored here
    // as all items in this struct will inherently be campaign missions.
    /// Path to the preview image for this map (from TMX `map_preview_image`).
    pub preview_image_path: String,

    /// The name of the location where the mission takes place (from TMX `location_name`).
    pub location_name: String,
    /// The address or more specific details of the mission location (from TMX `location_address`).
    pub location_address: String,
}

// Optional: A resource to hold all loaded campaign missions.
// This will be populated in Phase 3.
#[derive(Resource, Debug, Default, Clone)]
pub struct CampaignMissionsResource {
    pub missions: Vec<CampaignMissionData>,
}
