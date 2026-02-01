use crate::systems;
use bevy::prelude::*;

pub struct UnhaunterDifficultyPlugin;

impl Plugin for UnhaunterDifficultyPlugin {
    fn build(&self, app: &mut App) {
        systems::app_setup(app);
    }
}
