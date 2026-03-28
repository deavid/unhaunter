use crate::assets::CampaignAssets;
use crate::unified_mission_selection;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unorchestrator_core::UIContextState;

pub struct UnhaunterCampaignPlugin;

impl Plugin for UnhaunterCampaignPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(UIContextState::EngineBoot).load_collection::<CampaignAssets>(),
        );
        unified_mission_selection::app_setup(app);
        debug!("UnhaunterCampaignPlugin loaded.");
    }
}
