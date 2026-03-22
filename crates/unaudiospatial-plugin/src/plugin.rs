use crate::metrics;
use crate::systems::*;
use bevy::prelude::*;
use unaudiospatial_core::events::SoundEvent;
use untypes_core::states::AppState;

pub struct UnhaunterSpatialAudioPlugin;

impl Plugin for UnhaunterSpatialAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SoundEvent>()
            .add_systems(
                Update,
                spatial_audio_playback.run_if(in_state(AppState::InGame)),
            );

        metrics::register_all(app);
    }
}
