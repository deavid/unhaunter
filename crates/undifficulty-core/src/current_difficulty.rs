//! Current difficulty resource
//!
//! Holds the currently active difficulty settings.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use untypes_core::difficulty::Difficulty;

/// Resource that holds the current difficulty settings
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Resource, PartialEq, Default, Reflect)]
#[reflect(Resource, Default, PartialEq)]
pub struct CurrentDifficulty(pub Difficulty);

impl CurrentDifficulty {
    /// Creates a new `CurrentDifficulty` resource with the specified difficulty level.
    pub fn new(difficulty: Difficulty) -> Self {
        CurrentDifficulty(difficulty)
    }
}
