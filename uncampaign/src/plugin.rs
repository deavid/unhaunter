use crate::mission_selection;
use bevy::prelude::*;
// Removed unused import: uncore::states::AppState

pub struct UnhaunterCampaignPlugin;

impl Plugin for UnhaunterCampaignPlugin {
    fn build(&self, app: &mut App) {
        // We'll add systems here in later tasks
        mission_selection::app_setup(app); // Call setup from mission_selection module
        info!("UnhaunterCampaignPlugin loaded.");
    }
}
