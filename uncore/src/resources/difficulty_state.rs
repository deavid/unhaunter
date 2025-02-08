use bevy::prelude::*;

use crate::difficulty::Difficulty;

#[derive(Resource, Debug, Default)]
pub struct DifficultySelectionState {
    pub selected_difficulty: Difficulty,
    // Add this to store the selected map index
    pub selected_map_idx: usize,
}
