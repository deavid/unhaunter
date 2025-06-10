use crate::difficulty::Difficulty;
use bevy::prelude::*;
use bevy::utils::Instant;

#[derive(Resource, Debug)]
pub struct DifficultySelectionState {
    pub selected_difficulty: Difficulty,
    pub selected_map_idx: usize,
    pub state_entered_at: Instant,
}

impl Default for DifficultySelectionState {
    fn default() -> Self {
        Self {
            selected_difficulty: Difficulty::default(),
            selected_map_idx: 0,
            state_entered_at: Instant::now(),
        }
    }
}
