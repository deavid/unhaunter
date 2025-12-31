use bevy::prelude::*;
use unassets_core::resources::maps::Maps;
use unevents_core::events::hint::OnScreenHintEvent;
use unghost_core::resources::current_evidence_readings::CurrentEvidenceReadings;
use unmenu_core::mission_select::CurrentMissionSelectMode;
use unplayer_core::resources::PlayerInput;
use untypes_core::states::{AppState, GameState};

pub struct UnhaunterRootPlugin;

impl Plugin for UnhaunterRootPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_state::<GameState>()
            .init_resource::<Maps>()
            .add_systems(
                Startup,
                (
                    crate::assets_loading::load_assets,
                    crate::assets_loading::finish_loading,
                )
                    .chain(),
            );

        app.init_resource::<CurrentEvidenceReadings>();
        app.init_resource::<CurrentMissionSelectMode>();
        app.init_resource::<unnoise_core::perlin::PerlinNoise>();
        app.init_resource::<PlayerInput>();
        app.add_message::<OnScreenHintEvent>();

        crate::assets_loading::arch_setup::app_setup(app);
    }
}
