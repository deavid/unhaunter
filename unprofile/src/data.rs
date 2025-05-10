use bevy::{prelude::Resource, utils::HashMap};
use serde::{Deserialize, Serialize};
use uncore::types::grade::Grade;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProgressionData {
    pub bank: i64,
    #[serde(default)]
    pub insurance_deposit: i64,
    #[serde(default)]
    pub player_xp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AchievementData {
    #[serde(default)]
    pub expelled_first_ghost: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StatisticsData {
    #[serde(default)]
    pub total_missions_completed: u32,
    #[serde(default)]
    pub total_deaths: u32,
    #[serde(default)]
    pub total_play_time_seconds: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MapStatisticsData {
    #[serde(default)]
    pub total_missions_completed: u32,
    #[serde(default)]
    pub total_deaths: u32,
    #[serde(default)]
    pub total_play_time_seconds: f64,
    #[serde(default)]
    pub total_mission_completed_time_seconds: f64,
    #[serde(default)]
    pub best_score: i64,
    #[serde(default)]
    pub best_grade: Grade,
}

#[derive(Resource, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerProfileData {
    #[serde(default)]
    pub progression: ProgressionData,
    #[serde(default)]
    pub achievements: AchievementData,
    #[serde(default)]
    pub statistics: StatisticsData,
    #[serde(default)]
    pub map_statistics: HashMap<String, MapStatisticsData>,
}
