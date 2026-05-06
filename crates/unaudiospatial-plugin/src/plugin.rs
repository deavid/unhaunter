use crate::metrics;
use crate::systems::*;
use bevy::prelude::*;
use unaudiospatial_core::events::SoundEvent;
use unaudiospatial_core::listener::SpatialListener;
use uncommon_states_core::UIContextState;

pub struct UnhaunterSpatialAudioPlugin {
    pub enable: bool,
}

impl Plugin for UnhaunterSpatialAudioPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpatialListener>();
        app.add_message::<SoundEvent>();

        if self.enable {
            app.add_systems(
                Update,
                (
                    spatial_audio_playback,
                    monitor_audio_pileup,
                    process_audio_fadeouts,
                )
                    .run_if(in_state(UIContextState::InGame)),
            );
        }

        metrics::register_all(app);
    }
}
