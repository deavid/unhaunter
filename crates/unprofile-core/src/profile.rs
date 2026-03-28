use bevy::prelude::Resource;
use bevy_platform::collections::HashMap;
use serde::{Deserialize, Serialize};
use uncareer_core::grade::Grade;
use uncareer_core::progression::ProgressionData;
use uncareer_core::statistics::{MapStatisticsData, StatisticsData};
use undifficulty_core::difficulty::Difficulty;
use uninvestigation_core::evidence::Evidence;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct AchievementData {
    #[serde(default)]
    pub expelled_first_ghost: bool,
}

/// Statistics for a single walkie event
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct WalkieEventStats {
    /// Number of times this walkie event has been played
    #[serde(default)]
    pub play_count: u32,
    /// Total play time in seconds when this event was last played
    #[serde(default)]
    pub last_played_at_time: f64,
}

#[derive(Resource, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerProfileData {
    #[serde(default)]
    pub installation_id: Uuid,
    #[serde(default)]
    pub progression: ProgressionData,
    #[serde(default)]
    pub achievements: AchievementData,
    #[serde(default)]
    pub statistics: StatisticsData,
    #[serde(default)]
    pub map_statistics: HashMap<String, HashMap<Difficulty, MapStatisticsData>>,
    /// Tracks statistics for each walkie event, keyed by the event ID
    #[serde(default)]
    pub walkie_event_stats: HashMap<String, WalkieEventStats>,
    #[serde(default)]
    pub times_evidence_acknowledged_on_gear: HashMap<Evidence, u32>,
    #[serde(default)]
    pub times_evidence_acknowledged_in_journal: HashMap<Evidence, u32>,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeInstallationId(pub Uuid);

impl PlayerProfileData {
    /// Gets the best grade achieved for a specific map and difficulty.
    pub fn get_map_grade(&self, map_path: &str, difficulty: &Difficulty) -> Grade {
        self.map_statistics
            .get(map_path)
            .and_then(|stats| stats.get(difficulty))
            .filter(|s| s.total_missions_completed > 0)
            .map(|s| s.best_grade)
            .unwrap_or(Grade::NA)
    }
}
