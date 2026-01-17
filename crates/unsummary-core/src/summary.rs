use bevy::prelude::*;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfoundation_core::types::ghost::types::GhostType;
use unfoundation_core::types::grade::Grade;

#[derive(Debug, Clone, Resource, Default)]
pub struct SummaryData {
    pub time_taken_secs: f32,
    pub ghost_types: Vec<GhostType>,
    pub repellent_used_amt: u32,
    pub ghosts_unhaunted: u32,
    pub animated_final_score: i64,
    pub base_score: i64,
    pub difficulty_multiplier: f32,
    /// The multiplier based on the achieved grade (A, B, C, D, F)
    pub grade_multiplier: f64,
    pub difficulty: CurrentDifficulty,
    pub average_sanity: f32,
    pub player_count: usize,
    pub alive_count: usize,
    pub full_score: i64,

    /// Path to the map that was played.
    pub map_path: String,

    /// Indicates if the primary mission objectives were met.
    pub mission_successful: bool,

    /// The total money earned during the mission.
    pub money_earned: i64,

    /// The grade achieved for the mission.
    pub grade_achieved: Grade,

    /// The required deposit for the mission.
    pub required_deposit: i64,

    /// The base reward for completing the mission.
    pub mission_reward_base: i64,

    /// The amount of insurance deposit the player had at the start of mission
    pub deposit_originally_held: i64,

    /// The amount returned to bank after mission completion
    pub deposit_returned_to_bank: i64,

    /// Costs deducted from the deposit
    pub costs_deducted_from_deposit: i64,
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
            mission_successful: false, // Default to false
            ..default()
        }
    }

    pub fn calculate_score(&mut self, evaluator: &dyn MissionEvaluator) -> i64 {
        // Calculate base score using evaluator
        let base_score = evaluator.calculate_base_score(self);

        // Store the rounded base score
        self.base_score = base_score;

        // Store the difficulty multiplier
        let difficulty_multiplier = self.difficulty.0.difficulty_score_multiplier;
        self.difficulty_multiplier = difficulty_multiplier;

        // Apply difficulty multiplier to final score
        let score = (base_score as f32) * difficulty_multiplier;

        // Ensure score is within a reasonable range and return
        self.full_score = score.clamp(0.0, 1000000.0).round() as i64;
        self.full_score
    }
}
