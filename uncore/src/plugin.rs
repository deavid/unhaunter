use crate::events::hint::OnScreenHintEvent;
use crate::resources::current_evidence_readings::CurrentEvidenceReadings;
use crate::resources::hint_ui_state::HintUiState;
use crate::resources::mission_select_mode::CurrentMissionSelectMode;
use bevy::prelude::*;

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
        app.init_resource::<crate::resources::player_input::PlayerInput>();
        app.add_event::<OnScreenHintEvent>();
    }
}
