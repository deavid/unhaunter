use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unghost_core::resources::ghost_guess::GhostGuess;
use untruck_core::assets::TruckAssets;
use untruck_core::events::truck::TruckUIEvent;
use untruck_core::types::repellent_tracker::RepellentCraftTracker;
use untypes_core::states::AppState;

use super::loadoutui::EventButtonClicked;

pub struct UnhaunterTruckCorePlugin;

impl Plugin for UnhaunterTruckCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TruckUIEvent>()
            .add_message::<EventButtonClicked>()
            .init_resource::<GhostGuess>()
            .init_resource::<RepellentCraftTracker>();

        app.add_systems(
            OnEnter(AppState::InGame),
            super::systems::truck_ui_systems::init_repellent_tracker,
        );
        app.add_systems(
            OnExit(AppState::InGame),
            super::systems::truck_ui_systems::reset_repellent_tracker,
        );

        super::journal::app_setup_core(app);
        super::systems::in_truck_manager::app_setup(app);
    }
}

pub struct UnhaunterTruckPlugin;

impl Plugin for UnhaunterTruckPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::Loading).load_collection::<TruckAssets>(),
        );

        super::hydration::app_setup(app);
        super::evidence::app_setup(app);
        super::systems::setup::app_setup(app);
        super::ui::app_setup(app);
        super::journal::app_setup(app);
        super::sanity::app_setup(app);
        super::loadoutui::app_setup(app);
        super::truckgear::app_setup(app);
    }
}
