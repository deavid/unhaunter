use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use uncommon_states_core::UIContextState;
use unreplicon_core::resources::AuthorityRole;
use untruck_core::assets::TruckAssets;
use untruck_core::events::truck::TruckUIEvent;

pub struct UnhaunterTruckCorePlugin;

impl Plugin for UnhaunterTruckCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TruckUIEvent>();
        app.add_systems(
            Update,
            super::systems::truck_ui_systems::init_repellent_tracker
                .run_if(resource_exists::<AuthorityRole>),
        );
        super::journal::app_setup_core(app);
        super::systems::in_truck_manager::app_setup(app);
    }
}

pub struct UnhaunterTruckPlugin;

impl Plugin for UnhaunterTruckPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(UIContextState::EngineBoot).load_collection::<TruckAssets>(),
        );

        super::hydration::app_setup(app);
        super::systems::setup::app_setup(app);
        super::journal::app_setup(app);
        super::truckgear::app_setup(app);
    }
}
