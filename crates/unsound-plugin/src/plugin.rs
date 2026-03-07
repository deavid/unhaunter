use crate::metrics;
use crate::systems::*;
use bevy::prelude::*;
use untypes_core::roles::is_pure_client;
use untypes_core::states::AppState;

pub struct UnhaunterSoundPlugin;

impl Plugin for UnhaunterSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sound_update)
            .add_systems(Update, init_sound_grid)
            .add_systems(
                Update,
                crate::systems::handle_ghost_sound_field_broadcast.run_if(is_pure_client),
            )
            .add_systems(OnExit(AppState::InGame), reset_sound_grid);

        metrics::register_all(app);
    }
}
