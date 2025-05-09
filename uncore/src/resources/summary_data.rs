use crate::difficulty::CurrentDifficulty;
use crate::types::ghost::types::GhostType;
use bevy::prelude::*;

#[derive(Debug, Clone, Resource, Default)]
pub struct SummaryData {
    pub time_taken_secs: f32,
    pub ghost_types: Vec<GhostType>,
    pub repellent_used_amt: u32,
    pub ghosts_unhaunted: u32,
    pub final_score: i64,
    pub base_score: u32,
    pub difficulty_multiplier: f64,
    /// The multiplier based on the achieved grade (A, B, C, D, F)
    pub grade_multiplier: f64,
    pub difficulty: CurrentDifficulty,
    pub average_sanity: f32,
    pub player_count: usize,
    pub alive_count: usize,

    // NEW FIELDS for v1.0 Bank & Progression
    pub current_mission_id: String, // e.g., map path

    /// The total money earned during the mission.
    pub money_earned: i64,

    /// The grade achieved for the mission.
    pub grade_achieved: String,

    /// The required deposit for the mission.
    pub required_deposit: i64,

    /// The base reward for completing the mission.
    pub mission_reward_base: i64,

    /// The bonus reward for achieving a high grade.
    pub mission_reward_bonus: i64,

    /// The penalty applied for failing the mission.
    pub mission_penalty: i64,

    /// The amount of insurance deposit the player had at the start of mission
    pub deposit_originally_held: i64,

    /// The amount returned to bank after mission completion
    pub deposit_returned_to_bank: i64,

    /// Costs deducted from the deposit
    pub costs_deducted_from_deposit: i64,
}

impl SummaryData {
    pub fn new(ghost_types: Vec<GhostType>, difficulty: CurrentDifficulty) -> Self {
        Self {
            ghost_types,
            difficulty,
            ..default()
        }
    }

    pub fn calculate_score(&mut self) -> i64 {
        // Calculate base score without difficulty multiplier
        let mut base_score = (250.0 * self.ghosts_unhaunted as f64)
            / (1.0 + self.repellent_used_amt as f64)
            / (1.0 + (self.ghost_types.len() as u32 - self.ghosts_unhaunted) as f64);

        // Sanity modifier
        base_score *= (self.average_sanity as f64 + 30.0) / 50.0;

        // Store the difficulty multiplier
        let difficulty_multiplier = self.difficulty.0.difficulty_score_multiplier;

        // Apply additional multipliers
        let additional_multiplier = if self.player_count == self.alive_count {
            // Apply time bonus multiplier
            1.0 + 360.0 / (60.0 + self.time_taken_secs as f64)
        } else {
            self.alive_count as f64 / (self.player_count as f64 + 1.0)
        };

        // Apply additional multipliers to final score
        base_score *= additional_multiplier;

        // Calculate final score before time or survival bonuses
        let score = base_score.round() * difficulty_multiplier;

        // Store the rounded base score
        self.base_score = base_score.round() as u32;
        self.difficulty_multiplier = difficulty_multiplier;

        // Ensure score is within a reasonable range and return
        score.clamp(0.0, 1000000.0) as i64
    }
}
