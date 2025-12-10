use crate::difficulty::Difficulty;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionData {
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

    /// The fixed difficulty for this mission (parsed from TMX `campaign_difficulty`).
    pub difficulty: Difficulty,

    /// Whether this is a campaign mission or a custom mission.
    pub is_campaign_mission: bool,

    /// Path to the preview image for this map (from TMX `map_preview_image`).
    pub preview_image_path: String,

    /// The name of the location where the mission takes place (from TMX `location_name`).
    pub location_name: String,

    /// The address or more specific details of the mission location (from TMX `location_address`).
    pub location_address: String,

    /// The base reward for completing the mission.
    pub mission_reward_base: i64,

    /// The required deposit for the mission.
    pub required_deposit: i64,

    /// Score threshold for achieving grade A
    pub grade_a_score_threshold: i64,

    /// Score threshold for achieving grade B
    pub grade_b_score_threshold: i64,

    /// Score threshold for achieving grade C
    pub grade_c_score_threshold: i64,

    /// Score threshold for achieving grade D
    pub grade_d_score_threshold: i64,

    /// Minimum player level required for this map.
    pub min_player_level: i32,
}
