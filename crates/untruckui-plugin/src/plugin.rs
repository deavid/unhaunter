use bevy::prelude::*;

use super::loadoutui::EventButtonClicked;

pub struct UnhaunterTruckUIPlugin;

impl Plugin for UnhaunterTruckUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EventButtonClicked>();

        // TruckAssets are loaded by UnhaunterTruckPlugin in untruck-plugin.
        // We don't need to load them again here if they are already in the loading state.
        // However, to be safe and independent, we can add them to the loading state if not already there.
        // But since we are registered in app.rs alongside UnhaunterTruckPlugin, it's fine.

        super::ui::app_setup(app);
        super::sanity::app_setup(app);
        super::loadoutui::app_setup(app);
        super::evidence::app_setup(app);
        super::systems::app_setup(app);
        super::journal_blinking_system::app_setup(app);
        app.add_systems(
            Update,
            super::journal_ui_systems::button_system
                .run_if(in_state(untypes_core::states::GameState::Truck)),
        );
    }
}
