use crate::grade::Grade;
use bevy::prelude::*;
use undifficulty_core::difficulty::Difficulty;

/// Emitted by `uncareer-plugin` after grade and reward computation.
/// This is the canonical signal for the economy domain.
#[derive(Debug, Clone, Message)]
pub struct CareerRewardCalculatedEvent {
    pub money_earned: i64,
    pub xp_awarded: i64,
    pub deposit_refunded: i64,
    pub grade: Grade,
    pub map_path: String,
    pub difficulty: Difficulty,
    pub mission_successful: bool,
    pub time_taken_secs: f32,
}

/// Emitted by `uncareer-plugin` when the local player dies.
/// Triggers economic consequences (loss of deposit).
#[derive(Debug, Clone, Message)]
pub struct CareerDeathRecordedEvent {
    pub map_path: String,
    pub difficulty: Difficulty,
}
