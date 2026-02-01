use bevy::prelude::*;
use undifficulty_core::current_difficulty::CurrentDifficulty;

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<CurrentDifficulty>();
}
