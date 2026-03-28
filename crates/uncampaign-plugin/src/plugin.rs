use crate::assets::CampaignAssets;
use crate::unified_mission_selection;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use uncommon_app_core::states::AppState;

pub struct UnhaunterCampaignPlugin;

impl Plugin for UnhaunterCampaignPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::EngineBoot).load_collection::<CampaignAssets>(),
        );
        unified_mission_selection::app_setup(app);
        debug!("UnhaunterCampaignPlugin loaded.");
    }
}
