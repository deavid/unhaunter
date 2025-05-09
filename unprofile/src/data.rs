use bevy::prelude::Resource;
use bevy::utils::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProgressionData {
    pub bank: i64,
    #[serde(default)]
    pub insurance_deposit: i64,
    #[serde(default)]
    pub player_xp: i64,
    #[serde(default)]
    pub completed_missions: HashSet<String>,
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

#[derive(Resource, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerProfileData {
    #[serde(default)]
    pub progression: ProgressionData,
    #[serde(default)]
    pub achievements: AchievementData,
    #[serde(default)]
    pub statistics: StatisticsData,
}
