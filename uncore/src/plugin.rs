use crate::events::hint::OnScreenHintEvent;
use crate::resources::current_evidence_readings::CurrentEvidenceReadings;
use crate::resources::hint_ui_state::HintUiState;
use crate::resources::mission_select_mode::CurrentMissionSelectMode;
use crate::systems::board::sync_map_entity_field;
use crate::systems::evidence_decay::decay_evidence_clarity_system;
use bevy::prelude::*;

/// The core plugin for the Unhaunter game.
pub struct UnhaunterCorePlugin;

impl Plugin for UnhaunterCorePlugin {
    /// Builds the plugin by adding necessary systems to the app.
    fn build(&self, app: &mut App) {
        crate::metric_recorder::app_setup(app);
        app.init_resource::<CurrentMissionSelectMode>()
            .add_systems(Update, sync_map_entity_field);
        app.init_resource::<CurrentEvidenceReadings>();
        app.add_systems(Update, decay_evidence_clarity_system);
        app.init_resource::<HintUiState>();
        app.add_event::<OnScreenHintEvent>();
    }
}
