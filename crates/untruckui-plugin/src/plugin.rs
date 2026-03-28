use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use uncommon_app_core::states::AppState;
use uninput_core::states::InGameUiState;

use super::loadoutui::EventButtonClicked;
use crate::assets::TruckUiAssets;

pub struct UnhaunterTruckUIPlugin;

impl Plugin for UnhaunterTruckUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::EngineBoot).load_collection::<TruckUiAssets>(),
        );
        app.add_message::<EventButtonClicked>();

        super::ui::app_setup(app);
        super::sanity::app_setup(app);
        super::loadoutui::app_setup(app);
        super::evidence::app_setup(app);
        super::systems::app_setup(app);
        super::journal_blinking_system::app_setup(app);
        app.add_systems(
            Update,
            super::journal_ui_systems::button_system.run_if(in_state(InGameUiState::Truck)),
        );
    }
}
