use crate::metrics;
use crate::systems::*;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_seedling::nodes::itd::ItdNode;
use bevy_seedling::prelude::*;
use unaudiospatial_core::assets::MissionAssets;
use unaudiospatial_core::events::{LocalSoundEvent, SoundEvent};
use unaudiospatial_core::listener::SpatialListener;
use uncommon_states_core::UIContextState;

pub struct UnhaunterSpatialAudioPlugin {
    pub enable: bool,
}

#[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnSpatialPool;

fn setup_unspatial_pool(mut commands: Commands) {
    commands
        .spawn((
            SamplerPool(UnSpatialPool),
            Name::new("UnHaunter Spatial Pool"),
            sample_effects![
                VolumeNode::default(),
                SpatialBasicNode::default(),
                ItdNode::default()
            ],
            PoolSize(16..=64),
        ))
        .connect(SoundEffectsBus);
}

impl Plugin for UnhaunterSpatialAudioPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpatialListener>();
        app.add_message::<SoundEvent>();
        app.add_message::<LocalSoundEvent>();

        if self.enable {
            app.add_loading_state(
                LoadingState::new(UIContextState::EngineBoot).load_collection::<MissionAssets>(),
            );
            app.add_systems(Startup, setup_unspatial_pool);
            app.add_systems(Update, attach_flat_audio);
            app.add_systems(
                Update,
                (
                    local_spatial_audio_playback,
                    spatial_audio_playback,
                    update_spatial_audio,
                    monitor_audio_pileup,
                    process_audio_delayed_despawns,
                )
                    .run_if(in_state(UIContextState::InGame)),
            );
        }

        metrics::register_all(app);
    }
}
