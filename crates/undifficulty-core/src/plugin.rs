use crate::current_difficulty::CurrentDifficulty;
use bevy::prelude::*;

pub struct UnhaunterDifficultyPlugin;

impl Plugin for UnhaunterDifficultyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentDifficulty>();
    }
}
