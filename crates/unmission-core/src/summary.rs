use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use uninvestigation_core::ghost::GhostType;

use uncareer_core::grade::Grade;

/// Local per-player mission summary state.
///
/// This resource is computed and animated locally on player-bearing nodes.
/// It must never be replicated or attached to replicated entities.
/// Headless authority-only nodes do not own or build a summary.
#[derive(Debug, Clone, Resource, Default, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect(Resource, Default, PartialEq)]
pub struct SummaryData {
    pub time_taken_secs: f32,
    pub ghost_types: Vec<GhostType>,
    pub repellent_used_amt: u32,
    pub ghosts_unhaunted: u32,
    pub animated_final_score: i64,
    pub base_score: i64,
    pub difficulty_multiplier: f32,
    pub grade_multiplier: f64,
    pub difficulty: CurrentDifficulty,
    pub average_sanity: f32,
    pub player_count: usize,
    pub alive_count: usize,
    pub full_score: i64,
    pub map_path: String,
    pub mission_successful: bool,
    pub money_earned: i64,
    pub grade_achieved: Grade,
    pub required_deposit: i64,
    pub mission_reward_base: i64,
    pub deposit_originally_held: i64,
    pub deposit_returned_to_bank: i64,
    pub costs_deducted_from_deposit: i64,
    pub final_bank_total: i64,
}

pub trait MissionEvaluator: Send + Sync {
    fn calculate_base_score(&self, data: &SummaryData) -> i64;
    fn evaluate_grade(&self, score: i64) -> Grade;
}

#[derive(Resource)]
pub struct ActiveMissionEvaluator(pub Box<dyn MissionEvaluator>);

impl SummaryData {
    pub fn new(ghost_types: Vec<GhostType>, difficulty: CurrentDifficulty) -> Self {
        Self {
            ghost_types,
            difficulty,
            mission_successful: false,
            ..default()
        }
    }

    pub fn calculate_score(&mut self, evaluator: &dyn MissionEvaluator) -> i64 {
        let base_score = evaluator.calculate_base_score(self);
        self.base_score = base_score;
        let difficulty_multiplier = self.difficulty.0.difficulty_score_multiplier();
        self.difficulty_multiplier = difficulty_multiplier;
        let score = (base_score as f32) * difficulty_multiplier;
        self.full_score = score.clamp(0.0, 1000000.0).round() as i64;
        self.full_score
    }
}
