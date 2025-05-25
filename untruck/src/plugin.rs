use bevy::prelude::*;
use uncore::events::truck::TruckUIEvent;
use uncore::resources::ghost_guess::GhostGuess;

use super::loadoutui::EventButtonClicked;

pub struct UnhaunterTruckPlugin;

impl Plugin for UnhaunterTruckPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TruckUIEvent>()
            .add_event::<EventButtonClicked>()
            .init_resource::<GhostGuess>();

        super::evidence::app_setup(app);
        super::systems::app_setup(app);
        super::ui::app_setup(app);
        super::journal::app_setup(app);
        super::sanity::app_setup(app);
        super::loadoutui::app_setup(app);
    }
}
