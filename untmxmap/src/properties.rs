use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use tiled::Properties;

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct MapProps {
    pub name: String,
    pub author: String,
    pub description: String,
    pub music_path: String,
    pub mission_reward_base: i64,
    pub required_deposit: i64,
    pub grade_a_score_threshold: i64,
    pub grade_b_score_threshold: i64,
    pub grade_c_score_threshold: i64,
    pub grade_d_score_threshold: i64,
    #[serde(default)] // Defaults to 0 if not present
    pub min_player_level: i32,
}

impl MapProps {
    pub fn from_properties(properties: &Properties) -> Result<Self, String> {
        Ok(Self {
            name: tiled_utils::get_property_string(properties, "name")?,
            author: tiled_utils::get_property_string(properties, "author")?,
            description: tiled_utils::get_property_string(properties, "description")?,
            music_path: tiled_utils::get_property_string(properties, "music_path")?,
            mission_reward_base: tiled_utils::get_property_i64(properties, "mission_reward_base")?,
            required_deposit: tiled_utils::get_property_i64(properties, "required_deposit")?,
            grade_a_score_threshold: tiled_utils::get_property_i64(
                properties,
                "grade_a_score_threshold",
            )?,
            grade_b_score_threshold: tiled_utils::get_property_i64(
                properties,
                "grade_b_score_threshold",
            )?,
            grade_c_score_threshold: tiled_utils::get_property_i64(
                properties,
                "grade_c_score_threshold",
            )?,
            grade_d_score_threshold: tiled_utils::get_property_i64(
                properties,
                "grade_d_score_threshold",
            )?,
            min_player_level: tiled_utils::get_property_i32_optional(
                properties,
                "min_player_level",
            )?
            .unwrap_or(0),
        })
    }
}
