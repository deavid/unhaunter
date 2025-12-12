use bevy::prelude::*;
use uncore_events::hint::OnScreenHintEvent;
use uncore_resources::resources::current_evidence_readings::CurrentEvidenceReadings;
use uncore_resources::resources::hint_ui_state::HintUiState;
use uncore_resources::resources::mission_select_mode::CurrentMissionSelectMode;
use uncore_resources::resources::player_input::PlayerInput;

/// The core plugin for the Unhaunter game.
pub struct UnhaunterCorePlugin;

impl Plugin for UnhaunterCorePlugin {
    /// Builds the plugin by adding necessary systems to the app.
    fn build(&self, app: &mut App) {
        crate::metric_recorder::app_setup(app);
        crate::systems::evidence_decay::app_setup(app);
        crate::systems::board::app_setup(app);
        crate::systems::animation::app_setup(app);
        app.init_resource::<CurrentEvidenceReadings>();
        app.init_resource::<CurrentMissionSelectMode>();
        app.init_resource::<HintUiState>();
        app.init_resource::<crate::noise::PerlinNoise>();
        app.init_resource::<PlayerInput>();
        app.add_message::<OnScreenHintEvent>();
    }
}
