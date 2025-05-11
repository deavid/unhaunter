use crate::unified_mission_selection;
use bevy::prelude::*;
// Removed unused import: uncore::states::AppState

pub struct UnhaunterCampaignPlugin;

impl Plugin for UnhaunterCampaignPlugin {
    fn build(&self, app: &mut App) {
        // Register the unified mission selection system
        unified_mission_selection::app_setup(app); // Call setup from unified_mission_selection module
        info!("UnhaunterCampaignPlugin loaded.");
    }
}
