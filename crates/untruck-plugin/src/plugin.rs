use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use unevents_core::events::truck::TruckUIEvent;
use unghost_core::resources::ghost_guess::GhostGuess;
use untruck_core::assets::TruckAssets;
use untypes_core::states::AppState;

use super::loadoutui::EventButtonClicked;

pub struct UnhaunterTruckPlugin;

impl Plugin for UnhaunterTruckPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::Loading).load_collection::<TruckAssets>(),
        );
        app.add_message::<TruckUIEvent>()
            .add_message::<EventButtonClicked>()
            .init_resource::<GhostGuess>();

        super::evidence::app_setup(app);
        super::systems::app_setup(app);
        super::ui::app_setup(app);
        super::journal::app_setup(app);
        super::sanity::app_setup(app);
        super::loadoutui::app_setup(app);
        super::truckgear::app_setup(app);
    }
}
