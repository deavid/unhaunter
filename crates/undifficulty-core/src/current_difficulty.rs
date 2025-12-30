//! Current difficulty resource
//!
//! Holds the currently active difficulty settings.

use crate::Difficulty;
use crate::difficulty_settings::{DifficultySettings, DifficultyStruct};
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Resource that holds the current difficulty settings
#[derive(Debug, Clone, Serialize, Deserialize, Resource, PartialEq, Default)]
pub struct CurrentDifficulty(pub DifficultyStruct);

impl CurrentDifficulty {
    /// Creates a new `CurrentDifficulty` resource with the specified difficulty level.
    pub fn new(difficulty: Difficulty) -> Self {
        CurrentDifficulty(difficulty.as_struct())
    }
}

/// Returns the `DifficultyStruct` for the specified difficulty level.
pub fn get_difficulty_struct(difficulty: Difficulty) -> DifficultyStruct {
    difficulty.as_struct()
}
