use crate::unified_mission_selection;
use bevy::prelude::*;

pub struct UnhaunterCampaignPlugin;

impl Plugin for UnhaunterCampaignPlugin {
    fn build(&self, app: &mut App) {
        unified_mission_selection::app_setup(app);
        info!("UnhaunterCampaignPlugin loaded.");
    }
}
