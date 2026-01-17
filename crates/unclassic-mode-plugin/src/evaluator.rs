use unfoundation_core::types::grade::Grade;
use unsummary_core::summary::{MissionEvaluator, SummaryData};

pub struct ClassicEvaluator;

impl MissionEvaluator for ClassicEvaluator {
    fn calculate_base_score(&self, data: &SummaryData) -> i64 {
        // Calculate base score without difficulty multiplier
        let mut base_score = (250.0 * data.ghosts_unhaunted as f64)
            / (1.0 + data.repellent_used_amt as f64)
            / (1.0 + (data.ghost_types.len() as u32 - data.ghosts_unhaunted) as f64);

        // Sanity modifier
        base_score *= (data.average_sanity as f64 + 30.0) / 50.0;

        // Apply additional multipliers
        let additional_multiplier =
            if data.player_count == data.alive_count && data.player_count > 0 {
                // Apply time bonus multiplier
                1.0 + 360.0 / (60.0 + data.time_taken_secs as f64)
            } else if data.player_count > 0 {
                data.alive_count as f64 / (data.player_count as f64 + 1.0)
            } else {
                1.0
            };

        // Apply additional multipliers to final score
        base_score *= additional_multiplier;
        base_score.round() as i64
    }

    fn evaluate_grade(&self, score: i64) -> Grade {
        // Grade thresholds for Classic Mode.
        // In the future, these could be sourced from map data if passed to the evaluator.
        Grade::from_score(score, 1000, 800, 600, 400)
    }
}
