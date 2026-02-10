use bevy::prelude::Resource;
use bevy_platform::collections::HashMap;
use serde::{Deserialize, Serialize};
use unfoundation_core::types::grade::Grade;
use unghost_core::types::evidence::Evidence;
use untypes_core::difficulty::Difficulty;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProgressionData {
    pub bank: i64,
    #[serde(default)]
    pub insurance_deposit: i64,
    #[serde(default)]
    pub player_xp: i64,
    #[serde(default)]
    pub player_level: i32,
}

impl Default for ProgressionData {
    fn default() -> Self {
        Self {
            bank: 0,
            insurance_deposit: 0,
            player_xp: 0,
            player_level: 1,
        }
    }
}

impl ProgressionData {
    /// Calculates the player level based on player_xp.
    pub fn calculate_player_level(player_xp: i64) -> f64 {
        let n: f64 = 250.0;
        let k: f64 = 1.5;
        (player_xp as f64 / n + 1.0).powf(k.recip())
    }

    /// Updates the player_level field based on the current player_xp.
    pub fn update_level(&mut self) {
        self.player_level = Self::calculate_player_level(self.player_xp).floor() as i32;
    }

    /// Gets the current progress towards the next level as a float between 0.0 and 1.0.
    pub fn get_level_progress(&self) -> f32 {
        (Self::calculate_player_level(self.player_xp).fract()) as f32
    }

    /// Updates the player insurance deposit and bank balance.
    /// Returns an error message if the bank balance is insufficient.
    pub fn update_deposit(&mut self, required_deposit: i64) -> Result<(), String> {
        let additional_needed = required_deposit - self.insurance_deposit;

        match additional_needed.cmp(&0) {
            std::cmp::Ordering::Greater => {
                if self.bank >= additional_needed {
                    self.bank -= additional_needed;
                    self.insurance_deposit += additional_needed;
                } else {
                    return Err(format!(
                        "Insufficient Money in Bank for deposit. Required: ${}, Available: ${}",
                        required_deposit, self.bank
                    ));
                }
            }
            std::cmp::Ordering::Less => {
                let refund = -additional_needed;
                self.bank += refund;
                self.insurance_deposit -= refund;
            }
            std::cmp::Ordering::Equal => {}
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct AchievementData {
    #[serde(default)]
    pub expelled_first_ghost: bool,
}

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
