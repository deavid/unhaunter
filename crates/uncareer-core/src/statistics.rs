use serde::{Deserialize, Serialize};

use crate::grade::Grade;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct StatisticsData {
    #[serde(default)]
    pub total_missions_completed: u32,
    #[serde(default)]
    pub total_deaths: u32,
    #[serde(default)]
    pub total_play_time_seconds: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
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
