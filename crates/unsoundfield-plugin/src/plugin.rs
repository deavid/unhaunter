use crate::metrics;
use crate::systems::*;
use bevy::prelude::*;
use untypes_core::states::AppState;

pub struct UnhaunterSoundFieldPlugin;

impl Plugin for UnhaunterSoundFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sound_field_propagate)
            .add_systems(Update, init_sound_grid)
            .add_systems(OnExit(AppState::InGame), reset_sound_grid);

        metrics::register_all(app);
    }
}
