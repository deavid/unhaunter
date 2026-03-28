use crate::metrics;
use crate::systems::*;
use bevy::prelude::*;
use unaudiospatial_core::events::SoundEvent;
use unaudiospatial_core::listener::SpatialListener;
use unorchestrator_core::UIContextState;

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
                spatial_audio_playback.run_if(in_state(UIContextState::InGame)),
            );
        }

        metrics::register_all(app);
    }
}
