pub mod current_difficulty;
pub mod difficulty_settings;

mod difficulty_impl;

pub use current_difficulty::{CurrentDifficulty, get_difficulty_struct};
pub use difficulty_settings::{DifficultySettings, DifficultyStruct};
pub use uncore_types::types::difficulty::Difficulty;
